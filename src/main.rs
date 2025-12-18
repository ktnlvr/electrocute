#![feature(generic_const_exprs)]

use crate::parser::{CircuitBuilder, parse_commands, tokenize};

mod buffer;
mod circuit;
mod component;
mod expression;
mod numerical;
mod parser;
mod printing;
mod si;

pub fn main() {
    let netlist = include_str!("../sample.netlist");
    let tokens = tokenize(netlist);
    let cmds = parse_commands(tokens);

    for cmd in &cmds {
        println!("{:?}", cmd);
    }

    let mut builder = CircuitBuilder::new();
    builder.add_commands(cmds);

    let mut circuit = builder.build();

    const STEPS: usize = 100000;

    for _ in 0..STEPS {
        let dt = 0.01;

        circuit.stamp_all(dt);
        circuit.solve();
    }

    println!("{:?}", circuit.equations)
}
