use std::f64::consts::PI;

use bytemuck::{Pod, Zeroable};

use crate::{
    component::Component,
    numerical::{LinearEquations, c64},
};

#[derive(Debug, Pod, Zeroable, Clone, Copy, Default)]
#[repr(C)]
pub struct DC1Source {
    pub voltage_volt: f64,
}

impl Component for DC1Source {
    type State = ();
    const TERMINAL_COUNT: usize = 1;
    const PRIORITY: usize = 25;
    const PARAMETERS: &[&'static str] = &["V", "P"];

    fn stamp(&self, net: &mut LinearEquations, _: f64, [n]: [u32; 1], _: &Self::State) {
        net.clear_row(n);
        net.add_a(n, n, c64::ONE);
        net.set_b(n, c64::new(self.voltage_volt, 0.));
    }

    fn parameter(
        &self,
        _: &LinearEquations,
        _: [u32; Self::TERMINAL_COUNT],
        _: &Self::State,
        parameter: &str,
    ) -> Option<c64> {
        let v = c64::new(self.voltage_volt, 0.);
        match parameter {
            "V" => Some(v),
            _ => None,
        }
    }
}

#[derive(Debug, Pod, Zeroable, Clone, Copy, Default)]
#[repr(C)]
pub struct Ground;

impl Component for Ground {
    type State = ();
    const TERMINAL_COUNT: usize = 1;
    const PRIORITY: usize = 25;

    fn stamp(&self, net: &mut LinearEquations, _: f64, [n]: [u32; Self::TERMINAL_COUNT], _: &Self::State) {
        net.clear_row(n);
        net.add_a(n, n, c64::ONE);
        net.set_b(n, c64::ZERO);
    }
}

#[derive(Debug, Pod, Zeroable, Clone, Copy, Default)]
#[repr(C)]
pub struct AC1Source {
    pub amplitude_volt: f64,
    pub frequency_hz: f64,
    pub phase_rad: f64,
}

impl Component for AC1Source {
    type State = f64;
    const TERMINAL_COUNT: usize = 1;
    const PRIORITY: usize = 25;
    const PARAMETERS: &[&'static str] = &["V", "P", "f", "phi", "t"];

    fn stamp(&self, net: &mut LinearEquations, _: f64, [n]: [u32; 1], t: &Self::State) {
        net.clear_row(n);
        net.add_a(n, n, c64::ONE);

        let angle = 2.0 * PI * self.frequency_hz * t + self.phase_rad;
        let voltage = c64::polar(self.amplitude_volt, angle);

        net.set_b(n, voltage);
    }

    fn post_stamp(
        &self,
        _net: &LinearEquations,
        dt: f64,
        _terminals: [u32; Self::TERMINAL_COUNT],
        _state: &mut Self::State,
    ) {
        *_state += dt;
    }

    fn parameter(
        &self,
        _: &LinearEquations,
        _: [u32; Self::TERMINAL_COUNT],
        &t: &Self::State,
        parameter: &str,
    ) -> Option<c64> {
        let v = c64::new(self.amplitude_volt, 0.);
        match parameter {
            "V" => Some(v),
            "f" => Some(c64::new(self.frequency_hz, 0.)),
            "phi" => Some(c64::new(self.phase_rad, 0.)),
            "t" => Some(c64::new(t, 0.)),
            _ => None,
        }
    }
}
