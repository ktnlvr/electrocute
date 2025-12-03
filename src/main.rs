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

    const STEPS: usize = 10;

    let r1 = Resistor { resistance_ohm: 5. };
    let r2 = Resistor {
        resistance_ohm: 10.,
    };
    let r3 = Resistor {
        resistance_ohm: 15.,
    };

    let gnd = Ground;
    let src = DC1Source { voltage_volt: 5. };

    let mut circuit = Circuit::new();

    circuit
        .put(r1, [0, 1])
        .put(r2, [0, 1])
        .put(r3, [1, 2])
        .put(gnd, [0])
        .put(src, [2]);

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
