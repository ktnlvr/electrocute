use core::panic;
use std::collections::HashMap;

use crate::{
    circuit::Circuit,
    component::{AC1Source, Capacitor, DC1Source, Ground, Inductor, Resistor},
    numerical::c64,
    si::parse_si_number,
};

#[derive(Debug, Clone)]
pub enum Command {
    Component {
        component: String,
        name: Option<String>,
        terminals: Vec<String>,
        parameters: HashMap<String, c64>,
    },
}

pub fn tokenize(input: &str) -> Vec<Vec<String>> {
    input
        .split("\n")
        .filter_map(|s| Some(s.trim()).filter(|s| !s.is_empty() && !s.trim().starts_with("--")))
        .map(|s| s.split_ascii_whitespace().map(|s| s.to_owned()).collect())
        .collect()
}

fn parse_graph_arg(arg: &str) -> ((String, Option<String>), &str) {
    // Find '=' sign (if any) and split
    if let Some(eq_pos) = arg.find('=') {
        let (lhs, rhs) = arg.split_at(eq_pos);
        let lhs = lhs.trim();
        let rhs = &rhs[1..]; // skip '='
        let rhs = rhs.trim_start(); // keep rest including spaces for parse_expr

        // Split lhs on '_'
        let mut word_parts = lhs.splitn(2, '_');
        let main = word_parts.next().unwrap().to_string();
        let prefix = word_parts.next().map(|s| s.to_string());

        ((main, prefix), rhs)
    } else {
        // No '=' present, treat as previous
        let first_space = arg.find(' ').unwrap_or(arg.len());
        let (first_word, rest) = arg.split_at(first_space);

        let mut word_parts = first_word.splitn(2, '_');
        let main = word_parts.next().unwrap().to_string();
        let prefix = word_parts.next().map(|s| s.to_string());

        ((main, prefix), rest)
    }
}

pub fn parse_commands(tokens: Vec<Vec<String>>) -> Vec<Command> {
    let mut commands = Vec::new();

    for token_line in tokens {
        if token_line.is_empty() {
            continue;
        }

        let first_token = &token_line[0];

        let component_type = first_token.clone();
        let mut terminals = Vec::new();
        let mut parameters = HashMap::new();
        let mut component_name = None;

        for tok in token_line.iter().skip(1) {
            if tok.starts_with("\"") {
                let Some((name, tail)) = tok.split_at(1).1.split_once("\"") else {
                    panic!("Invalid component name format");
                };
                assert!(tail.is_empty());
                component_name = Some(name.to_string());
            } else if let Some(eq_pos) = tok.find('=') {
                let key = &tok[..eq_pos];
                let value = &tok[eq_pos + 1..];
                if let Some(val) = parse_si_number(value) {
                    parameters.insert(key.to_string(), c64::new(val, 0.));
                }
            } else {
                terminals.push(tok.clone());
            }
        }

        commands.push(Command::Component {
            component: component_type,
            name: component_name,
            terminals,
            parameters,
        });
    }

    commands
}

pub struct CircuitBuilder {
    commands: Vec<Command>,
}

impl CircuitBuilder {
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
        }
    }

    pub fn add_commands(&mut self, cmds: Vec<Command>) {
        self.commands.extend(cmds);
    }

    pub fn build(&self) -> Circuit {
        let mut circuit = Circuit::new();
        let mut terminal_map: HashMap<String, u32> = HashMap::new();
        let mut next_index: u32 = 0;

        for command in &self.commands {
            if let Command::Component {
                component,
                name,
                terminals,
                parameters,
            } = command
            {
                let mut term_indices = Vec::new();
                for t in terminals {
                    let idx = *terminal_map.entry(t.clone()).or_insert_with(|| {
                        let i = next_index;
                        next_index += 1;
                        i
                    });
                    term_indices.push(idx);
                }

                match component.as_str() {
                    "dc-source-1-terminal" => {
                        let mut comp = DC1Source::default();
                        if let Some(&v) = parameters.get("V") {
                            comp.voltage_volt = v.re;
                        }
                        circuit.put_raw(comp, term_indices.try_into().unwrap(), name.clone());
                    }
                    "resistor" => {
                        let mut comp = Resistor::default();
                        if let Some(&r) = parameters.get("R") {
                            comp.resistance_ohm = r.re;
                        }
                        circuit.put_raw(comp, term_indices.try_into().unwrap(), name.clone());
                    }
                    "ground" => {
                        let comp = Ground::default();
                        circuit.put_raw(comp, term_indices.try_into().unwrap(), name.clone());
                    }
                    "capacitor" => {
                        let mut comp = Capacitor::default();
                        if let Some(&c) = parameters.get("C") {
                            comp.capacitance_f = c.re;
                        }
                        circuit.put_raw(comp, term_indices.try_into().unwrap(), name.clone());
                    }
                    "inductor" => {
                        let mut comp = Inductor::default();
                        if let Some(&c) = parameters.get("L") {
                            comp.inductance_h = c.re;
                        }
                        circuit.put_raw(comp, term_indices.try_into().unwrap(), name.clone());
                    }
                    "ac-source-1-terminal" => {
                        let mut comp = AC1Source::default();
                        if let Some(&c) = parameters.get("A") {
                            comp.amplitude_volt = c.re;
                        }
                        if let Some(&c) = parameters.get("f") {
                            comp.frequency_hz = c.re;
                        }
                        if let Some(&c) = parameters.get("phi") {
                            comp.phase_rad = c.re;
                        }
                        circuit.put_raw(comp, term_indices.try_into().unwrap(), name.clone());
                    }
                    _ => panic!("Unknown component type: {}", component),
                }
            }
        }

        circuit
    }
}
