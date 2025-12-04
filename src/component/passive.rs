use bytemuck::{Pod, Zeroable};

use crate::{
    component::Component,
    net::{Net, c64},
};

#[derive(Debug, Pod, Zeroable, Clone, Copy, Default)]
#[repr(C)]
pub struct Resistor {
    pub resistance_ohm: f64,
}

impl Component for Resistor {
    type State = ();
    const TERMINAL_COUNT: usize = 2;
    const PRIORITY: usize = 10;
    const PARAMETERS: &[&'static str] = &["R", "V", "I"];

    fn stamp(&self, net: &mut Net, _: f64, [n1, n2]: [u32; 2], _: &Self::State) {
        let y = c64::new(1. / self.resistance_ohm, 0.);

        net.add_a(n1, n1, y);
        net.add_a(n1, n2, -y);
        net.add_a(n2, n1, -y);
        net.add_a(n2, n2, y);
    }

    fn parameter(
        &self,
        net: &Net,
        [start, end]: [u32; Self::TERMINAL_COUNT],
        _: &Self::State,
        parameter: &str,
    ) -> Option<c64> {
        let r = c64::new(self.resistance_ohm, 0.);
        let v = net.get_voltage_across(start, end);

        match parameter {
            "R" => Some(r),
            "V" => Some(v),
            "I" => Some(v / r),
            _ => None,
        }
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
    const PARAMETERS: &[&'static str] = &["C", "V", "I"];

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

    fn parameter(
        &self,
        net: &Net,
        [start, end]: [u32; Self::TERMINAL_COUNT],
        state: &Self::State,
        parameter: &str,
    ) -> Option<c64> {
        let v = net.get_voltage_across(start, end);
        let g_eq = c64::new(self.capacitance_f, 0.) / c64::new(1., 0.);
        let i = g_eq * c64::new(state.previous_voltage_re, state.previous_voltage_im);
        match parameter {
            "C" => Some(c64::new(self.capacitance_f, 0.)),
            "V" => Some(v),
            "I" => Some(i),
            _ => None,
        }
    }
}
