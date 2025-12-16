use bytemuck::Pod;

mod passive;
mod sources;

pub use passive::*;
pub use sources::*;

use crate::numerical::{Numbers, c64};

pub trait Component: Pod {
    type State: Pod + Clone + Copy + Default;
    const TERMINAL_COUNT: usize;
    const PRIORITY: usize;
    const PARAMETERS: &[&'static str] = &[];

    fn stamp(
        &self,
        net: &mut Numbers,
        dt: f64,
        terminals: [u32; Self::TERMINAL_COUNT],
        state: &Self::State,
    );

    fn post_stamp(
        &self,
        _net: &Numbers,
        _dt: f64,
        _terminals: [u32; Self::TERMINAL_COUNT],
        _state: &mut Self::State,
    ) {
    }

    fn parameter(
        &self,
        _net: &Numbers,
        _terminals: [u32; Self::TERMINAL_COUNT],
        _state: &Self::State,
        _parameter: &str,
    ) -> Option<c64> {
        None
    }
}
