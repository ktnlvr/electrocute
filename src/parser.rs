use std::collections::HashMap;

use crate::{
    ComponentLibrary,
    circuit::Circuit,
    expression::{Expression, parse_expr},
};

#[derive(Debug, Clone)]
pub enum Command {
    Component {
        component: String,
        name: Option<String>,
        terminals: Vec<String>,
        parameters: HashMap<String, Expression>,
    },
}

fn parse_identifier(input: &str) -> Option<(&str, &str)> {
    let input = input.trim_start();

    let mut chars = input.char_indices();

    let mut end = 0;

    if let Some((i, c)) = chars.next() {
        if c.is_ascii_alphabetic() || c == '_' || c.is_ascii_punctuation() {
            end = i + c.len_utf8();
        } else {
            return None;
        }
    } else {
        return None;
    }

    for (i, c) in chars {
        if c.is_ascii_alphanumeric() || c == '_' || c.is_ascii_punctuation() {
            end = i + c.len_utf8();
        } else {
            break;
        }
    }

    let (ident, rest) = input.split_at(end);

    Some((ident, rest))
}

fn parse_quoted(input: &str) -> (Option<&str>, &str) {
    let input = input.trim_start();
    if input.is_empty() {
        return (None, "");
    }

    if !input.starts_with("\"") {
        return (None, input);
    }

    let Some((identifier, rest)) = input[1..].split_once("\"") else {
        return (None, input);
    };

    (Some(identifier), rest)
}

fn parse_equality(input: &str) -> Option<(Expression, Option<&str>, &str)> {
    let rest = input.trim_start();
    if rest.is_empty() {
        return None;
    }

    let (name, rest) = if let Some((var, rest)) = parse_identifier(input) {
        (Some(var), rest)
    } else {
        (None, rest)
    };

    let rest = rest.trim_start();
    if !rest.starts_with("=") {
        return None;
    }

    let rest = &rest[1..];

    let Some((expr, rest)) = parse_expr(rest).ok() else {
        return None;
    };

    Some((expr, name, rest))
}

pub fn parse_commands<'line>(
    library: &ComponentLibrary,
    lines: impl Iterator<Item = &'line str>,
) -> Vec<Command> {
    let mut commands = Vec::new();

    for line in lines {
        let Some((component, rest)) = parse_identifier(line) else {
            continue;
        };

        let rest = rest.trim_start();
        let (name, mut rest) = parse_quoted(rest);

        let Some(terminal_count) = library.terminal_count_of(component) else {
            unreachable!()
        };

        let mut terminals = vec![];
        for _ in 0..terminal_count {
            let Some((identifier, new_rest)) = parse_identifier(rest) else {
                unreachable!()
            };

            terminals.push(identifier.to_string());

            rest = new_rest.trim_start();
        }

        let mut parameters = HashMap::new();
        while let Some((identifier, new_rest)) = rest.split_once("=") {
            let Ok((expr, new_rest)) = parse_expr(new_rest.trim_start()) else {
                break;
            };

            parameters.insert(identifier.to_string(), expr);

            rest = new_rest.trim_start();
        }

        commands.push(Command::Component {
            component: component.to_string(),
            name: name.map(String::from),
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
        todo!()
    }
}
