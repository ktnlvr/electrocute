use std::collections::HashMap;

use crate::{
    circuit::Circuit,
    component::{Capacitor, DC1Source, Ground, Resistor},
    si::parse_si_number,
};

pub fn tokenize(input: &str) -> Vec<Vec<String>> {
    input
        .split("\n")
        .filter_map(|s| Some(s.trim()).filter(|s| !s.is_empty()))
        .map(|s| s.split_ascii_whitespace().map(|s| s.to_owned()).collect())
        .collect()
}

pub fn generate_circuit(tokens: Vec<Vec<String>>) -> Circuit {
    let mut circuit = Circuit::new();
    let mut terminal_map: HashMap<String, u32> = HashMap::new();
    let mut next_index: u32 = 0;

    for token_line in tokens {
        if token_line.is_empty() {
            continue;
        }

        let component_type = &token_line[0];
        let mut terminals: Vec<u32> = Vec::new();
        let mut inner_params: HashMap<String, f64> = HashMap::new();

        for tok in token_line.iter().skip(1) {
            if let Some(eq_pos) = tok.find('=') {
                let key = &tok[..eq_pos];
                let value = &tok[eq_pos + 1..];
                if let Some(val) = parse_si_number(value) {
                    inner_params.insert(key.to_string(), val);
                }
            } else {
                let idx = *terminal_map.entry(tok.clone()).or_insert_with(|| {
                    let i = next_index;
                    next_index += 1;
                    i
                });
                terminals.push(idx);
            }
        }

        match component_type.as_str() {
            "dc-source-1-terminal" => {
                let mut comp = DC1Source::default();
                if let Some(&v) = inner_params.get("V") {
                    comp.voltage_volt = v;
                }
                circuit.put(comp, terminals.try_into().unwrap());
            }
            "resistor" => {
                let mut comp = Resistor::default();
                if let Some(&r) = inner_params.get("R") {
                    comp.resistance_ohm = r;
                }
                circuit.put(comp, terminals.try_into().unwrap());
            }
            "ground" => {
                let comp = Ground::default();
                circuit.put(comp, terminals.try_into().unwrap());
            }
            "capacitor" => {
                let mut comp = Capacitor::default();
                if let Some(&c) = inner_params.get("C") {
                    comp.capacitance_f = c;
                }
                circuit.put(comp, terminals.try_into().unwrap());
            }
            _ => panic!("Unknown component type: {}", component_type),
        }
    }

    circuit
}
