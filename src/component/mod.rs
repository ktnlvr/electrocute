use std::collections::HashMap;

use bytemuck::Pod;

mod passive;
mod sources;

pub use passive::*;
pub use sources::*;

use crate::{
    circuit::Circuit,
    expression::Expression,
    numerical::{LinearEquations, c64},
};

pub trait Component: Pod {
    type State: Pod + Clone + Copy + Default;
    const TERMINAL_COUNT: usize;
    const PRIORITY: usize;
    const PARAMETERS: &[&'static str] = &[];
    const ACTIVE_TERMINALS: &[(usize, usize)] = &[(0, 0)];

    fn stamp(
        &self,
        le: &mut LinearEquations,
        dt: f64,
        terminals: [u32; Self::TERMINAL_COUNT],
        state: &Self::State,
    );

    fn post_stamp(
        &self,
        _le: &LinearEquations,
        _dt: f64,
        _terminals: [u32; Self::TERMINAL_COUNT],
        _state: &mut Self::State,
    ) {
    }

    fn parameter(
        &self,
        _le: &LinearEquations,
        _terminals: [u32; Self::TERMINAL_COUNT],
        _state: &Self::State,
        _parameter: &str,
    ) -> Option<c64> {
        None
    }
}

pub struct ComponentLibrary {
    constructors: HashMap<
        String,
        Box<
            dyn Fn(
                &mut Circuit,
                HashMap<String, Expression>,
            ) -> Result<HashMap<String, Expression>, Vec<ComponentError>>,
        >,
    >,
    terminal_counts: HashMap<String, usize>,
}

pub struct MissingRequiredParameter {
    pub parameter: String,
}

pub enum ComponentError {
    UnusedSuppliedParameter { parameter: String },
    MissingRequiredParameter { parameter: String },
}

impl ComponentLibrary {
    pub fn new() -> Self {
        Self {
            constructors: Default::default(),
            terminal_counts: Default::default(),
        }
    }

    pub fn register_component<C: Component>(
        &mut self,
        name: impl ToString,
        constructor: impl Fn(
            HashMap<String, Expression>,
        )
            -> Result<(C, HashMap<String, Expression>), Vec<MissingRequiredParameter>>,
    ) -> &mut Self {
        let name = name.to_string();

        self.terminal_counts
            .insert(name.to_owned(), C::TERMINAL_COUNT);

        self.constructors
            .insert(name, Box::new(|circuit, hashmap| todo!()));

        self
    }

    pub fn terminal_count_of(&self, component_name: &str) -> Option<usize> {
        self.terminal_counts.get(component_name).copied()
    }
}
