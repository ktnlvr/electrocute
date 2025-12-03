use std::{any::TypeId, collections::HashMap};

use bytemuck::{bytes_of, try_from_bytes, try_from_bytes_mut};

use crate::{component::Component, net::Net};

struct Components {
    buffer: Vec<u8>,
    solve_fn: Box<dyn Fn(&[u8], &mut Net, f64, &[u32], &mut [u8])>,
    comp_size: usize,
    state_size: usize,
    terminals_count: usize,
}

pub struct Circuit {
    circuit: HashMap<TypeId, Components>,
}

impl Circuit {
    pub fn new() -> Self {
        Self {
            circuit: Default::default(),
        }
    }

    pub fn put<T: 'static + Component>(
        &mut self,
        component: T,
        terminals: [u32; T::N],
    ) -> &mut Self {
        let type_id = TypeId::of::<T>();
        let slice = bytes_of(&component);

        let mut buffer = vec![];

        buffer.extend_from_slice(slice);

        if T::is_stateful() {
            buffer.extend_from_slice(bytes_of(&T::State::default()));
        }

        for terminal in terminals {
            buffer.extend_from_slice(bytes_of(&terminal));
        }

        self.circuit.entry(type_id).or_insert(Components {
            buffer,
            solve_fn: Box::new(
                |this: &[u8], net: &mut Net, dt: f64, ts: &[u32], state: &mut [u8]| {
                    let this: &T = try_from_bytes(this).unwrap();
                    let mut terminals = [0u32; T::N];
                    terminals
                        .iter_mut()
                        .zip(ts.iter())
                        .for_each(|(t, tt)| *t = *tt);
                    let state: &mut T::State = try_from_bytes_mut(state).unwrap();

                    this.solve(net, dt, terminals, state);
                },
            ),
            comp_size: std::mem::size_of::<T>(),
            state_size: if T::is_stateful() {
                std::mem::size_of::<T::State>()
            } else {
                0
            },
            terminals_count: T::N,
        });

        self
    }

    pub fn fill_in_net(&mut self, net: &mut Net, dt: f64) {
        for c in self.circuit.values_mut() {
            let comp_size = c.comp_size;
            let state_size = c.state_size;

            let bytes = &mut c.buffer[..];

            let (comp_bytes, rest) = bytes.split_at_mut(comp_size);
            let (state_bytes, terminal_bytes) = rest.split_at_mut(state_size);

            let (_, terminals, _) = unsafe { terminal_bytes.align_to::<u32>() };

            (c.solve_fn)(comp_bytes, net, dt, terminals, state_bytes);
        }
    }
}
