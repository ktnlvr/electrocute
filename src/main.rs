use crate::{
    net::{Net, c64},
    parser::tokenize,
    strategy::{DC1Source, Ground, Resistor, SolvingStrategy},
};

mod net;
mod parser;
mod strategy;

pub fn main() {
    let netlist = include_str!("../sample.netlist");
    let components = tokenize(netlist);

    println!("{:?}", components);

    let resistor_solver =
        SolvingStrategy::<Resistor, 2>::new("resistor", |net, _, this, [n1, n2], ()| {
            let y = c64::new(1. / this.resistance_ohm, 0.);

            net.add_jacobian(n1, n1, y);
            net.add_jacobian(n1, n2, -y);
            net.add_jacobian(n2, n1, -y);
            net.add_jacobian(n2, n2, y);
        });

    let dc_source_solver =
        SolvingStrategy::<DC1Source, 1>::new("dc-source-1-terminal", |net, _, this, [n], ()| {
            net.clear_row_jacobian(n);
            net.add_jacobian(n, n, c64::ONE);
            net.set_current(n, c64::new(this.voltage_volt, 0.));
        });

    let ground_solver = SolvingStrategy::<Ground, 1>::new("ground", |net, _, this, [n], ()| {
        net.clear_row_jacobian(n);
        net.add_jacobian(n, n, c64::ONE);
        net.set_current(n, c64::ZERO);
    });

    const STEPS: usize = 100;

    let r1 = Resistor {
        resistance_ohm: 500.,
    };
    let r2 = Resistor {
        resistance_ohm: 1000.,
    };
    let gnd = Ground;
    let src = DC1Source { voltage_volt: 5. };

    for step in 0..STEPS {
        let mut net = Net::new(3);
        let dt = 0.01;

        (resistor_solver.solve)(&mut net, dt, &r1, [1, 2], ());
        (resistor_solver.solve)(&mut net, dt, &r2, [1, 2], ());
        (dc_source_solver.solve)(&mut net, dt, &src, [2], ());
        (ground_solver.solve)(&mut net, dt, &gnd, [0], ());

        net.solve();

        println!("{step}: {:?}", net.voltages);
    }
}
