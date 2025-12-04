use bytemuck::{Pod, Zeroable};

use crate::net::{Net, c64};

pub trait Component: Pod {
    type State: Pod + Clone + Copy + Default;
    const TERMINAL_COUNT: usize;
    const PRIORITY: usize;

    fn stamp(
        &self,
        net: &mut Net,
        dt: f64,
        terminals: [u32; Self::TERMINAL_COUNT],
        state: &mut Self::State,
    );

    fn describe(
        &self,
        net: &Net,
        terminals: [u32; Self::TERMINAL_COUNT],
        state: &Self::State,
    ) -> Vec<(&'static str, c64)>;
}

#[derive(Debug, Pod, Zeroable, Clone, Copy, Default)]
#[repr(C)]
pub struct Resistor {
    pub resistance_ohm: f64,
}

impl Component for Resistor {
    type State = ();
    const TERMINAL_COUNT: usize = 2;
    const PRIORITY: usize = 10;

    fn stamp(&self, net: &mut Net, _: f64, [n1, n2]: [u32; 2], _: &mut Self::State) {
        let y = c64::new(1. / self.resistance_ohm, 0.);

        net.add_a(n1, n1, y);
        net.add_a(n1, n2, -y);
        net.add_a(n2, n1, -y);
        net.add_a(n2, n2, y);
    }

    fn describe(
        &self,
        net: &Net,
        [start, end]: [u32; 2],
        _: &Self::State,
    ) -> Vec<(&'static str, c64)> {
        let r = c64::new(self.resistance_ohm, 0.);
        let v = net.get_voltage_across(start, end);
        vec![("R", r), ("V", v), ("I", v / r)]
    }
}

#[derive(Debug, Pod, Zeroable, Clone, Copy, Default)]
#[repr(C)]
pub struct DC1Source {
    pub voltage_volt: f64,
}

impl Component for DC1Source {
    type State = ();
    const TERMINAL_COUNT: usize = 1;
    const PRIORITY: usize = 25;

    fn stamp(&self, net: &mut Net, _: f64, [n]: [u32; 1], _: &mut Self::State) {
        net.clear_row_jacobian(n);
        net.add_a(n, n, c64::ONE);
        net.set_b(n, c64::new(self.voltage_volt, 0.));
    }

    fn describe(
        &self,
        net: &Net,
        terminals: [u32; Self::TERMINAL_COUNT],
        state: &Self::State,
    ) -> Vec<(&'static str, c64)> {
        vec![("V", c64::new(self.voltage_volt, 0.))]
    }
}

#[derive(Debug, Pod, Zeroable, Clone, Copy, Default)]
#[repr(C)]
pub struct Ground;

impl Component for Ground {
    type State = ();
    const TERMINAL_COUNT: usize = 1;
    const PRIORITY: usize = 25;

    fn stamp(&self, net: &mut Net, _: f64, [n]: [u32; Self::TERMINAL_COUNT], _: &mut Self::State) {
        net.clear_row_jacobian(n);
        net.add_a(n, n, c64::ONE);
        net.set_b(n, c64::ZERO);
    }

    fn describe(
        &self,
        net: &Net,
        terminals: [u32; Self::TERMINAL_COUNT],
        state: &Self::State,
    ) -> Vec<(&'static str, c64)> {
        vec![]
    }
}
