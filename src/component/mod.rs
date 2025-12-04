use bytemuck::Pod;

use crate::net::{Net, c64};

mod passive;
mod sources;

pub use passive::*;
pub use sources::*;

pub trait Component: Pod {
    type State: Pod + Clone + Copy + Default;
    const TERMINAL_COUNT: usize;
    const PRIORITY: usize;
    const PARAMETERS: &[&'static str] = &[];

    fn stamp(
        &self,
        net: &mut Net,
        dt: f64,
        terminals: [u32; Self::TERMINAL_COUNT],
        state: &Self::State,
    );

    fn post_stamp(
        &self,
        _net: &Net,
        _dt: f64,
        _terminals: [u32; Self::TERMINAL_COUNT],
        _state: &mut Self::State,
    ) {
    }

    fn parameter(
        &self,
        _net: &Net,
        _terminals: [u32; Self::TERMINAL_COUNT],
        _state: &Self::State,
        _parameter: &str,
    ) -> Option<c64> {
        None
    }
}
