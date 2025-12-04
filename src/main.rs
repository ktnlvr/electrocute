#![feature(generic_const_exprs)]

use crate::{
    net::Net,
    parser::{CircuitBuilder, generate_commands, tokenize},
    printing::print_table,
};

mod circuit;
mod component;
mod net;
mod parser;
mod printing;
mod si;

pub fn main() {
    let netlist = include_str!("../sample.netlist");
    let tokens = tokenize(netlist);
    let cmds = generate_commands(tokens);

    for cmd in &cmds {
        println!("{:?}", cmd);
    }

    let mut builder = CircuitBuilder::new();
    builder.add_commands(cmds);

    let mut circuit = builder.build();

    const STEPS: usize = 10000;

    let mut net = Net::new(3);
    for _ in 0..STEPS {
        let dt = 0.01;

        net.reset();
        circuit.stamp(&mut net, dt);
        net.solve();
    }

    let (headers, rows) = circuit.describe(&net);
    println!("{}", print_table(headers, rows));
}
