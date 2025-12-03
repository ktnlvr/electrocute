use bytemuck::{Pod, Zeroable};

use crate::net::{Net, c64};

#[derive(Debug, Pod, Zeroable, Clone, Copy)]
#[repr(C)]
pub struct Resistor {
    pub resistance_ohm: f64,
}

impl Component for Resistor {
    type State = ();
}

#[derive(Debug, Pod, Zeroable, Clone, Copy)]
#[repr(C)]
pub struct DC1Source {
    pub voltage_volt: f64,
}

impl Component for DC1Source {
    type State = ();
}

#[derive(Debug, Pod, Zeroable, Clone, Copy)]
#[repr(C)]
pub struct Ground;

impl Component for Ground {
    type State = ();
}

pub trait Component: Pod {
    type State: Pod + Clone + Copy;
}

pub struct SolvingStrategy<T: Component, const N: usize> {
    pub name: String,
    pub solve: Box<dyn Fn(&mut Net, f64, &T, [u32; N], T::State) -> T::State>,
}

impl<T: Component, const N: usize> SolvingStrategy<T, N> {
    pub fn new(
        name: impl ToString,
        solve: impl Fn(&mut Net, f64, &T, [u32; N], T::State) -> T::State + 'static,
    ) -> Self {
        Self {
            name: name.to_string(),
            solve: Box::new(solve),
        }
    }
}
