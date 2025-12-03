use std::any::TypeId;

use bytemuck::{Pod, Zeroable};

use crate::net::{Net, c64};

pub trait Component: Pod {
    type State: Pod + Clone + Copy + Default;
    const N: usize;

    fn solve(&self, net: &mut Net, dt: f64, terminals: [u32; Self::N], state: &mut Self::State);

    fn is_stateful() -> bool {
        TypeId::of::<Self::State>() != TypeId::of::<()>()
    }

    fn describe(
        &self,
        net: &mut Net,
        terminals: [u32; Self::N],
        state: &mut Self::State,
    ) -> Vec<(&'static str, c64)>;
}

#[derive(Debug, Pod, Zeroable, Clone, Copy)]
#[repr(C)]
pub struct Resistor {
    pub resistance_ohm: f64,
}

impl Component for Resistor {
    type State = ();
    const N: usize = 2;

    fn solve(&self, net: &mut Net, _: f64, [n1, n2]: [u32; 2], _: &mut Self::State) {
        let y = c64::new(1. / self.resistance_ohm, 0.);

        net.add_jacobian(n1, n1, y);
        net.add_jacobian(n1, n2, -y);
        net.add_jacobian(n2, n1, -y);
        net.add_jacobian(n2, n2, y);
    }

    fn describe(
        &self,
        net: &mut Net,
        [start, end]: [u32; 2],
        _: &mut Self::State,
    ) -> Vec<(&'static str, c64)> {
        vec![
            ("R", c64::new(self.resistance_ohm, 0.)),
            ("V", net.get_voltage_across(start, end)),
        ]
    }
}

#[derive(Debug, Pod, Zeroable, Clone, Copy)]
#[repr(C)]
pub struct DC1Source {
    pub voltage_volt: f64,
}

impl Component for DC1Source {
    type State = ();
    const N: usize = 1;

    fn solve(&self, net: &mut Net, _: f64, [n]: [u32; 1], _: &mut Self::State) {
        net.clear_row_jacobian(n);
        net.add_jacobian(n, n, c64::ONE);
        net.set_current(n, c64::new(self.voltage_volt, 0.));
    }

    fn describe(
        &self,
        net: &mut Net,
        terminals: [u32; Self::N],
        state: &mut Self::State,
    ) -> Vec<(&'static str, c64)> {
        vec![("V", c64::new(self.voltage_volt, 0.))]
    }
}

#[derive(Debug, Pod, Zeroable, Clone, Copy)]
#[repr(C)]
pub struct Ground;

impl Component for Ground {
    type State = ();
    const N: usize = 1;

    fn solve(&self, net: &mut Net, _: f64, [n]: [u32; Self::N], _: &mut Self::State) {
        net.clear_row_jacobian(n);
        net.add_jacobian(n, n, c64::ONE);
        net.set_current(n, c64::ZERO);
    }

    fn describe(
        &self,
        net: &mut Net,
        terminals: [u32; Self::N],
        state: &mut Self::State,
    ) -> Vec<(&'static str, c64)> {
        vec![]
    }
}
