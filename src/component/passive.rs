use bytemuck::{Pod, Zeroable};

use crate::{
    component::Component,
    numerical::{Numbers, c64},
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
    const PARAMETERS: &[&'static str] = &["R", "V", "I", "P"];

    fn stamp(&self, net: &mut Numbers, _: f64, [n1, n2]: [u32; 2], _: &Self::State) {
        let y = c64::new(1. / self.resistance_ohm, 0.);

        net.add_a(n1, n1, y);
        net.add_a(n1, n2, -y);
        net.add_a(n2, n1, -y);
        net.add_a(n2, n2, y);
    }

    fn parameter(
        &self,
        net: &Numbers,
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
            "P" => Some(v * v / r),
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
    v_old_re: f64,
    v_old_im: f64,
    dv_per_dt_re: f64,
    dv_per_dt_im: f64,
}

impl Component for Capacitor {
    type State = CapacitorState;

    const TERMINAL_COUNT: usize = 2;
    const PRIORITY: usize = 10;
    const PARAMETERS: &[&'static str] = &["C", "V", "I", "P"];

    fn stamp(
        &self,
        net: &mut Numbers,
        dt: f64,
        [n1, n2]: [u32; Self::TERMINAL_COUNT],
        state: &Self::State,
    ) {
        let g_eq = c64::new(self.capacitance_f / dt, 0.);
        let v_prev = c64::new(state.v_old_re, state.v_old_im);
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
        net: &Numbers,
        dt: f64,
        [n1, n2]: [u32; Self::TERMINAL_COUNT],
        state: &mut Self::State,
    ) {
        let v = net.get_voltage_across(n1, n2);
        state.dv_per_dt_re = (v.re - state.v_old_re) / dt;
        state.dv_per_dt_im = (v.im - state.v_old_im) / dt;
        state.v_old_re = v.re;
        state.v_old_im = v.im;
    }

    fn parameter(
        &self,
        net: &Numbers,
        [start, end]: [u32; Self::TERMINAL_COUNT],
        state: &Self::State,
        parameter: &str,
    ) -> Option<c64> {
        let v = net.get_voltage_across(start, end);
        let v_prev = c64::new(state.v_old_re, state.v_old_im);
        let dv_per_dt = c64::new(state.dv_per_dt_re, state.dv_per_dt_im);

        let g_eq = c64::new(self.capacitance_f, 0.) / c64::new(1., 0.);
        let i = g_eq * v_prev;

        match parameter {
            "C" => Some(c64::new(self.capacitance_f, 0.)),
            "V" => Some(v),
            "I" => Some(i),
            "P" => Some(v * i * dv_per_dt),
            _ => None,
        }
    }
}

#[derive(Pod, Zeroable, Clone, Copy, Default)]
#[repr(C)]
pub struct Inductor {
    pub inductance_h: f64,
}

#[derive(Pod, Zeroable, Clone, Copy, Default)]
#[repr(C)]
pub struct InductorState {
    i_old_re: f64,
    i_old_im: f64,
    di_per_dt_re: f64,
    di_per_dt_im: f64,
}

impl Component for Inductor {
    type State = InductorState;

    const TERMINAL_COUNT: usize = 2;
    const PRIORITY: usize = 10;
    const PARAMETERS: &[&'static str] = &["L", "V", "I", "P"];

    fn stamp(
        &self,
        net: &mut Numbers,
        dt: f64,
        [n1, n2]: [u32; Self::TERMINAL_COUNT],
        state: &Self::State,
    ) {
        let g_eq = c64::new(dt / self.inductance_h, 0.);
        let i_prev = c64::new(state.i_old_re, state.i_old_im);

        let i_hist = i_prev;

        net.add_a(n1, n1, g_eq);
        net.add_a(n1, n2, -g_eq);
        net.add_a(n2, n1, -g_eq);
        net.add_a(n2, n2, g_eq);

        net.add_b(n1, -i_hist);
        net.add_b(n2, i_hist);
    }

    fn post_stamp(
        &self,
        net: &Numbers,
        dt: f64,
        [n1, n2]: [u32; Self::TERMINAL_COUNT],
        state: &mut Self::State,
    ) {
        let v = net.get_voltage_across(n1, n2);

        let di_dt_re = v.re / self.inductance_h;
        let di_dt_im = v.im / self.inductance_h;

        let i_new_re = state.i_old_re + di_dt_re * dt;
        let i_new_im = state.i_old_im + di_dt_im * dt;

        state.di_per_dt_re = di_dt_re;
        state.di_per_dt_im = di_dt_im;
        state.i_old_re = i_new_re;
        state.i_old_im = i_new_im;
    }

    fn parameter(
        &self,
        net: &Numbers,
        [start, end]: [u32; Self::TERMINAL_COUNT],
        state: &Self::State,
        parameter: &str,
    ) -> Option<c64> {
        let v = net.get_voltage_across(start, end);
        let i_prev = c64::new(state.i_old_re, state.i_old_im);

        match parameter {
            "L" => Some(c64::new(self.inductance_h, 0.)),
            "V" => Some(v),
            "I" => Some(i_prev),
            "P" => Some(v * i_prev),
            _ => None,
        }
    }
}
