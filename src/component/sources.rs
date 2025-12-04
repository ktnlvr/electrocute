use bytemuck::{Pod, Zeroable};

use crate::{
    component::Component,
    net::{Net, c64},
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

    fn stamp(&self, net: &mut Net, _: f64, [n]: [u32; 1], _: &Self::State) {
        net.clear_row_jacobian(n);
        net.add_a(n, n, c64::ONE);
        net.set_b(n, c64::new(self.voltage_volt, 0.));
    }

    fn parameter(
        &self,
        _: &Net,
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

    fn stamp(&self, net: &mut Net, _: f64, [n]: [u32; Self::TERMINAL_COUNT], _: &Self::State) {
        net.clear_row_jacobian(n);
        net.add_a(n, n, c64::ONE);
        net.set_b(n, c64::ZERO);
    }
}
