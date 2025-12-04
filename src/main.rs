#![feature(generic_const_exprs)]

use crate::{
    net::Net,
    parser::{generate_circuit, tokenize},
};

mod circuit;
mod component;
mod net;
mod parser;
mod si;

pub fn main() {
    let netlist = include_str!("../sample.netlist");
    let tokens = tokenize(netlist);

    let mut circuit = generate_circuit(tokens);

    const STEPS: usize = 10000;

    let mut net = Net::new(3);
    for _ in 0..STEPS {
        let dt = 0.01;

        net.reset();
        circuit.stamp(&mut net, dt);
        net.solve();
    }

    circuit.describe(&net);
}
