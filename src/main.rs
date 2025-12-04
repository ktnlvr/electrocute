#![feature(generic_const_exprs)]

use std::collections::HashMap;

use crate::{
    net::{Net, c64},
    parser::{CircuitBuilder, Command, generate_commands, tokenize},
    printing::{print_chart, print_table},
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

    let mut graphing = HashMap::<((String, String), (String, String)), Vec<(c64, c64)>>::new();
    for cmd in &cmds {
        let Command::Graph { x, y } = cmd else {
            continue;
        };

        graphing.insert((x.clone(), y.clone()), vec![]);
    }

    let mut builder = CircuitBuilder::new();
    builder.add_commands(cmds);

    let mut circuit = builder.build();

    const STEPS: usize = 10;

    let mut net = Net::new(3);
    for _ in 0..STEPS {
        let dt = 0.01;

        net.reset();
        circuit.stamp(&mut net, dt);
        net.solve();

        for (((y_comp, y_param), (x_comp, x_param)), series) in &mut graphing {
            let x = circuit.describe_component(&net, &x_comp, &x_param);
            let y = circuit.describe_component(&net, &y_comp, &y_param);
            series.push((x, y));
        }
    }

    let (headers, rows) = circuit.describe(&net);
    println!("{}", print_table(headers, rows));

    for (((y_comp, y_param), (x_comp, x_param)), series) in &graphing {
        let points = series
            .iter()
            .map(|(x, y)| (x.norm(), y.norm()))
            .collect::<Vec<_>>();

        let chart = print_chart(
            format!("{}_{} against {}_{}", x_comp, x_param, y_comp, y_param),
            points,
        );

        println!("{}", chart);
    }
}
