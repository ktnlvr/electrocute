use std::collections::HashMap;

use crate::{
    circuit::Circuit,
    expression::{Expression},
};

#[derive(Debug, Clone)]
pub enum Command {
    Comment(String),
    Component {
        component: String,
        name: Option<String>,
        terminals: Vec<String>,
        parameters: HashMap<String, Expression>,
    },
}

pub struct Parser {
    pos: usize,
    advancements: Vec<usize>,
    chars: Vec<char>,
}

impl<S> From<S> for Parser
where
    S: ToString,
{
    fn from(value: S) -> Self {
        let chars: Vec<_> = value.to_string().chars().collect();

        Parser {
            pos: 0,
            advancements: vec![],
            chars,
        }
    }
}

impl Parser {
    pub fn advance_pop(&mut self) {
        if let Some(pos) = self.advancements.pop() {
            self.pos = pos;
        }
    }

    pub fn advance_drop(&mut self) {
        self.advancements.pop();
    }

    pub fn advance_push(&mut self) {
        self.advancements.push(self.pos);
    }

    pub fn skip_whitespace(&mut self) {
        while !self.is_eof() && self.chars[self.pos].is_whitespace() {
            self.pos += 1;
        }
    }

    pub fn is_eof(&self) -> bool {
        self.pos >= self.chars.len()
    }

    pub fn expect(&mut self, pred: impl FnOnce(char) -> bool) -> Option<char> {
        if self.is_eof() {
            return None;
        }

        let ch = self.chars[self.pos];
        if pred(ch) {
            self.pos += 1;
            Some(ch)
        } else {
            None
        }
    }

    pub fn expect_char(&mut self, ch: char) -> bool {
        self.expect(|c| c == ch).is_some()
    }

    pub fn parse_identifier(&mut self) -> Option<String> {
        self.advance_push();

        let Some(c) = self.expect(|c| c.is_ascii_alphabetic() || c == '-') else {
            self.advance_pop();
            return None;
        };

        let mut chars = vec![c];

        while let Some(c) = self.expect(|c| c.is_ascii_alphanumeric() || c == '-') {
            chars.push(c);
        }

        Some(chars.into_iter().collect())
    }

    pub fn parse_string(&mut self) -> Option<String> {
        self.advance_push();

        if !self.expect_char('"') {
            self.advance_pop();
            return None;
        }

        let mut chars = vec![];

        while let Some(c) = self.expect(|c| c != '"') {
            chars.push(c);
        }

        if !self.expect_char('"') {
            self.advance_pop();
            return None;
        }

        self.advance_drop();
        Some(chars.into_iter().collect())
    }

    pub fn parse_number_k(&mut self) -> Option<i64> {
        let mut value = 0;
        let mut found = false;

        while let Some(c) = self.expect(|c| c.is_ascii_digit()) {
            found = true;
            value = value * 10 + (c as i64 - '0' as i64);
        }

        if !found || !self.expect_char('k') {
            return None;
        }

        Some(value * 1_000)
    }

    pub fn parse_comment(&mut self) -> Option<String> {
        self.advance_push();

        if !self.expect_char('-') || !self.expect_char('-') {
            self.advance_pop();
            return None;
        }

        let mut chars = Vec::new();

        while let Some(c) = self.expect(|c| c != '\n') {
            chars.push(c);
        }

        self.advance_drop();
        Some(chars.into_iter().collect::<String>().trim().to_string())
    }


    pub fn parse_commands(&mut self) -> Option<Vec<Command>>{
        let mut commands = vec![];
        
        self.advance_push();

        loop {
            self.advance_drop();
            self.skip_whitespace();
            self.advance_push();

            if let Some(comment) = self.parse_comment() {
                commands.push(Command::Comment(comment));
                self.advance_drop();
                continue;
            }

            if let Some(command) = self.parse_component_command() {
                commands.push(command);
                self.advance_drop();
                continue;
            }

            break;
        }

        self.advance_drop();
        Some(commands)
    }

    pub fn parse_component_command(&mut self) -> Option<Command> {
        self.advance_push();

        let Some(kind) = self.parse_identifier() else {
            self.advance_pop();
            return None;
        };

        self.skip_whitespace();

        let name = self.parse_string();
        if name.is_some() {
            self.skip_whitespace();
        }

        let mut terminals = Vec::new();
        let mut params = HashMap::new();
        let mut parsing_params = false;

        loop {
            self.skip_whitespace();

            let Some(key) = self.parse_identifier() else {
                break;
            };

            if self.expect_char('=') {
                parsing_params = true;

                let value = self.parse_number_k().expect("number followed by K");

                params.insert(key, Expression::Real(value as f64));
            } else {
                if parsing_params {
                    self.advance_pop();
                    break;
                }

                terminals.push(key);
            }
        }

        self.advance_drop();

        Some(Command::Component {
            component: kind,
            name,
            terminals,
            parameters: params.into_iter().collect(),
        })
    }
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
