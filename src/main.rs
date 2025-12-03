#![feature(generic_const_exprs)]

use crate::{
    circuit::Circuit,
    component::{DC1Source, Ground, Resistor},
    net::Net,
    parser::tokenize,
};

mod circuit;
mod component;
mod net;
mod parser;

pub fn main() {
    let netlist = include_str!("../sample.netlist");
    let components = tokenize(netlist);

    println!("{:?}", components);

    const STEPS: usize = 100;

    let r1 = Resistor {
        resistance_ohm: 500.,
    };
    let r2 = Resistor {
        resistance_ohm: 1000.,
    };
    let gnd = Ground;
    let src = DC1Source { voltage_volt: 3. };

    let mut circuit = Circuit::new();

    circuit
        .put(r1, [0, 1])
        .put(r2, [0, 1])
        .put(gnd, [0])
        .put(src, [1]);

    let mut net = Net::new(2);
    for _ in 0..STEPS {
        let dt = 0.01;

        circuit.fill_in_net(&mut net, dt);
        net.solve();
    }

    println!("{:?}", net.voltages);

    circuit.describe(&mut net);
}
