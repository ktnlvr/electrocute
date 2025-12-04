#![feature(generic_const_exprs)]

use std::collections::HashMap;

use crate::{
    net::{Net, c64},
    parser::{CircuitBuilder, Command, parse_commands, tokenize},
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
    let cmds = parse_commands(tokens);

    for cmd in &cmds {
        println!("{:?}", cmd);
    }

    let mut graphing =
        HashMap::<((Option<String>, String), (Option<String>, String)), Vec<(c64, c64)>>::new();
    for cmd in &cmds {
        let Command::Graph { x, y } = cmd else {
            continue;
        };

        graphing.insert((x.clone(), y.clone()), vec![]);
    }

    let mut builder = CircuitBuilder::new();
    builder.add_commands(cmds);

    let mut circuit = builder.build();

    const STEPS: usize = 100;

    let mut net = Net::new(3);
    let mut total_t = 0.;
    for _ in 0..STEPS {
        let dt = 0.01;

        net.reset();
        circuit.stamp(&mut net, dt);
        net.solve();

        for (((y_comp, y_param), (x_comp, x_param)), series) in &mut graphing {
            let x = x_comp
                .as_ref()
                .map(|x_comp| circuit.get_component_parameter(&net, &x_comp, x_param))
                .unwrap_or_else(|| {
                    if x_param == "t" {
                        c64::new(total_t, 0.)
                    } else {
                        todo!()
                    }
                });

            let y = y_comp
                .as_ref()
                .map(|y_comp| circuit.get_component_parameter(&net, &y_comp, y_param))
                .unwrap_or_else(|| {
                    if y_param == "t" {
                        c64::new(total_t, 0.)
                    } else {
                        todo!()
                    }
                });

            series.push((x, y));
        }

        total_t += dt;
    }

    let (headers, rows) = circuit.describe(&net);
    println!("{}", print_table(headers, rows));

    for series in graphing.values() {
        let points = series
            .iter()
            .map(|(x, y)| (x.norm(), y.norm()))
            .collect::<Vec<_>>();

        let chart = print_chart("", points);

        println!("{}", chart);
    }
}
