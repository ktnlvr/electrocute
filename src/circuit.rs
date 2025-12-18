use std::{any::TypeId, collections::HashMap};

use crate::{buffer::ComponentBuffer, component::Component, numerical::LinearEquations};

struct Components {
    buffer: ComponentBuffer,
    terminals: Vec<u32>,
    stamp_all_fn: Box<dyn Fn(&ComponentBuffer, &mut LinearEquations, f64, &[u32])>,
    post_stamp_all_fn: Box<dyn Fn(&mut ComponentBuffer, &LinearEquations, f64, &[u32])>,
}

pub struct Circuit {
    names: HashMap<(TypeId, u32), String>,
    circuit: HashMap<TypeId, Components>,
    equations: LinearEquations,
}

impl Circuit {
    pub fn new() -> Self {
        Self {
            circuit: Default::default(),
            equations: LinearEquations::from_coordinates([]),
            names: Default::default(),
        }
    }

    pub fn put_raw<C: Component>(
        &mut self,
        component: C,
        name: Option<String>,
        terminals: [u32; C::TERMINAL_COUNT],
    ) {
        let type_id = TypeId::of::<C>();

        let components = self.circuit.entry(type_id).or_insert_with(|| Components {
            buffer: ComponentBuffer::new::<C>(),
            terminals: vec![],
            stamp_all_fn: Box::new(|components, le, dt, terminals| {
                components
                    .iter::<C>()
                    .enumerate()
                    .for_each(|(i, (c, state))| {
                        let start = C::TERMINAL_COUNT * i;
                        let end = C::TERMINAL_COUNT * (i + 1);
                        c.stamp(le, dt, terminals[start..end].try_into().unwrap(), state);
                    });
            }),
            post_stamp_all_fn: Box::new(|components, le, dt, terminals| {
                let mut terminal = [0; C::TERMINAL_COUNT];
                for (i, &v) in terminals.iter().enumerate() {
                    terminal[i] = v;
                }

                components.iter_mut::<C>().for_each(|(c, state)| {
                    c.post_stamp(le, dt, terminal, state);
                });
            }),
        });

        let idx = components.buffer.len() as u32;

        components.buffer.push(component);
        components.terminals.extend_from_slice(&terminals);

        if let Some(name) = name {
            self.names.insert((type_id, idx), name);
        }
    }

    pub fn stamp_all(&mut self, dt: f64) {
        for (_, component) in &mut self.circuit {
            (component.stamp_all_fn)(
                &component.buffer,
                &mut self.equations,
                dt,
                &component.terminals[..],
            );
        }

        for (_, component) in &mut self.circuit {
            (component.post_stamp_all_fn)(
                &mut component.buffer,
                &self.equations,
                dt,
                &component.terminals[..],
            );
        }
    }

    pub fn solve(&mut self) {
        self.equations.solve();
    }
}
