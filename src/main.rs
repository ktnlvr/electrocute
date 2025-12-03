#![feature(generic_const_exprs)]

use crate::{
    circuit::Circuit,
    component::{DC1Source, Ground, Resistor},
    net::Net,
    parser::{generate_circuit, tokenize},
};

mod circuit;
mod component;
mod net;
mod parser;

pub fn main() {
    let netlist = include_str!("../sample.netlist");
    let tokens = tokenize(netlist);

    let mut circuit = generate_circuit(tokens);

    const STEPS: usize = 10;

    let mut net = Net::new(3);
    for _ in 0..STEPS {
        let dt = 0.01;

        net.reset();
        circuit.fill_in_net(&mut net, dt);
        println!("{}", net.jacobian);

        net.solve();
        println!("{:?}", net.voltages);
        circuit.describe(&net);
    }
}
