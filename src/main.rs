#![feature(generic_const_exprs)]

use electrocute::Parser;

use crate::{
    component::{ComponentLibrary, DC1Source, Ground, Resistor},
    parser::{CircuitBuilder},
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

    let mut p = Parser::from(netlist);

    let c = p.parse_commands();
    
    println!("{:#?}", c);
}
