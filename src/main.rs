#![feature(generic_const_exprs)]

use crate::{
    component::{ComponentLibrary, DC1Source, Ground, Resistor},
    parser::{CircuitBuilder, parse_commands},
};

mod buffer;
mod circuit;
mod component;
mod expression;
mod numerical;
mod parser;
mod printing;
mod si;

pub fn main() {
    let mut components = ComponentLibrary::new();

    components
        .register_component::<DC1Source>("dc-source-1-terminal", |_| todo!())
        .register_component::<Resistor>("resistor", |_| todo!())
        .register_component::<Ground>("ground", |_| todo!());

    let netlist = include_str!("../sample.netlist");
    let cmds = parse_commands(&components, netlist.split("\n"));

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
