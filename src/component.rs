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
        state: &Self::State,
    );

    fn post_stamp(
        &self,
        net: &Net,
        dt: f64,
        terminals: [u32; Self::TERMINAL_COUNT],
        state: &mut Self::State,
    ) {
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

#[derive(Debug, Pod, Zeroable, Clone, Copy, Default)]
#[repr(C)]
pub struct Resistor {
    pub resistance_ohm: f64,
}

impl Component for Resistor {
    type State = ();
    const TERMINAL_COUNT: usize = 2;
    const PRIORITY: usize = 10;

    fn stamp(&self, net: &mut Net, _: f64, [n1, n2]: [u32; 2], _: &Self::State) {
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

    fn stamp(&self, net: &mut Net, _: f64, [n]: [u32; 1], _: &Self::State) {
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

    fn stamp(&self, net: &mut Net, _: f64, [n]: [u32; Self::TERMINAL_COUNT], _: &Self::State) {
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

#[derive(Pod, Zeroable, Clone, Copy, Default)]
#[repr(C)]
pub struct Capacitor {
    pub capacitance_f: f64,
}

#[derive(Pod, Zeroable, Clone, Copy, Default)]
#[repr(C)]
pub struct CapacitorState {
    previous_voltage_re: f64,
    previous_voltage_im: f64,
}

impl Component for Capacitor {
    type State = CapacitorState;

    const TERMINAL_COUNT: usize = 2;

    const PRIORITY: usize = 10;

    fn stamp(
        &self,
        net: &mut Net,
        dt: f64,
        [n1, n2]: [u32; Self::TERMINAL_COUNT],
        state: &Self::State,
    ) {
        let g_eq = c64::new(self.capacitance_f / dt, 0.);
        let v_prev = c64::new(state.previous_voltage_re, state.previous_voltage_im);
        let i_hist = g_eq * v_prev;

        net.add_a(n1, n1, g_eq);
        net.add_a(n1, n2, -g_eq);
        net.add_a(n2, n1, -g_eq);
        net.add_a(n2, n2, g_eq);

        net.add_b(n1, i_hist);
        net.add_b(n2, -i_hist);
    }

    fn post_stamp(
        &self,
        net: &Net,
        _: f64,
        [n1, n2]: [u32; Self::TERMINAL_COUNT],
        state: &mut Self::State,
    ) {
        let v = net.get_voltage_across(n1, n2);
        state.previous_voltage_re = v.re;
        state.previous_voltage_im = v.im;
    }

    fn describe(
        &self,
        net: &Net,
        [n1, n2]: [u32; Self::TERMINAL_COUNT],
        state: &Self::State,
    ) -> Vec<(&'static str, c64)> {
        let v = net.get_voltage_across(n1, n2);
        let g_eq = c64::new(self.capacitance_f, 0.) / c64::new(1., 0.); // For info only
        let i = g_eq * c64::new(state.previous_voltage_re, state.previous_voltage_im);
        vec![("C", c64::new(self.capacitance_f, 0.)), ("V", v), ("I", i)]
    }
}
