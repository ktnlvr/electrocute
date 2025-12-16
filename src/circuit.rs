use std::{any::TypeId, collections::HashMap, mem::size_of};

use bytemuck::{
    bytes_of,
    checked::{try_from_bytes, try_from_bytes_mut},
};

use crate::{component::Component, numerical::Numbers};

struct Components {
    names: Vec<Option<String>>,
    buffer: Vec<u8>,
    stamp_fn: Box<dyn Fn(&[u8], &mut Numbers, f64, &[u32], &[u8])>,
    post_stamp_fn: Box<dyn Fn(&[u8], &Numbers, f64, &[u32], &mut [u8])>,
    component_size: usize,
    state_size: usize,
    priority: usize,
    terminals: usize,
}

pub struct Circuit {
    circuit: HashMap<TypeId, Components>,
    numbers: Numbers,
}

impl Circuit {
    pub fn new() -> Self {
        Self {
            circuit: Default::default(),
            numbers: Numbers::new(2),
        }
    }

    pub fn put_raw<T: Component>(
        &mut self,
        component: T,
        terminals: [u32; T::TERMINAL_COUNT],
        name: Option<String>,
    ) -> &mut Self {
        let components = self.circuit.entry(TypeId::of::<T>()).or_insert(Components {
            names: vec![],
            buffer: vec![],
            stamp_fn: Box::new(
                |this: &[u8], net: &mut Numbers, dt: f64, ts: &[u32], state: &[u8]| {
                    let this: &T = try_from_bytes(this).unwrap();
                    let state: &T::State = try_from_bytes(state).unwrap();

                    let mut terminals = [0u32; T::TERMINAL_COUNT];
                    ts.iter().enumerate().for_each(|(i, t)| {
                        terminals[i] = *t;
                    });

                    this.stamp(net, dt, terminals, state);
                },
            ),
            post_stamp_fn: Box::new(
                |this: &[u8], net: &Numbers, dt: f64, ts: &[u32], state: &mut [u8]| {
                    let this: &T = try_from_bytes(this).unwrap();
                    let state: &mut T::State = try_from_bytes_mut(state).unwrap();

                    let mut terminals = [0u32; T::TERMINAL_COUNT];
                    ts.iter().enumerate().for_each(|(i, t)| {
                        terminals[i] = *t;
                    });

                    this.post_stamp(net, dt, terminals, state);
                },
            ),
            component_size: size_of::<T>(),
            state_size: size_of::<T::State>(),
            terminals: T::TERMINAL_COUNT,
            priority: T::PRIORITY,
        });

        components.names.push(name);
        components.buffer.extend(bytes_of(&component));
        components
            .buffer
            .extend_from_slice(bytes_of(&T::State::default()));

        (0..T::TERMINAL_COUNT).for_each(|i| {
            components.buffer.extend(bytes_of(&terminals[i]));
        });

        self
    }

    pub fn stamp_all(&mut self, dt: f64) {
        let mut values = self.circuit.iter_mut().collect::<Vec<_>>();
        values.sort_by_key(|(_, c)| c.priority);
        let values = values
            .into_iter()
            .map(|(c, _)| c)
            .copied()
            .collect::<Vec<_>>();

        for i in values.clone() {
            let components = self.circuit.get_mut(&i).unwrap();

            let total_size = components.component_size
                + components.state_size
                + components.terminals * std::mem::size_of::<u32>();

            if total_size == 0 {
                continue;
            }

            let mut offset = 0;
            while offset + total_size <= components.buffer.len() {
                let (slice, _) = components.buffer[offset..].split_at_mut(total_size);

                let (comp_bytes, rest) = slice.split_at_mut(components.component_size);
                let (state_bytes, term_bytes) = rest.split_at_mut(components.state_size);

                let comp_bytes: &[u8] = comp_bytes;
                let mut terminals = [0u32; 8];
                for i in 0..components.terminals {
                    let start = i * 4;
                    let end = start + 4;
                    terminals[i] = u32::from_ne_bytes(term_bytes[start..end].try_into().unwrap());
                }

                (components.stamp_fn)(
                    comp_bytes,
                    &mut self.numbers,
                    dt,
                    &terminals[..components.terminals],
                    state_bytes,
                );

                offset += total_size;
            }
        }

        for i in values.clone() {
            let components = self.circuit.get_mut(&i).unwrap();

            let total_size = components.component_size
                + components.state_size
                + components.terminals * std::mem::size_of::<u32>();

            if total_size == 0 {
                continue;
            }

            let mut offset = 0;
            while offset + total_size <= components.buffer.len() {
                let (slice, _) = components.buffer[offset..].split_at_mut(total_size);

                let (comp_bytes, rest) = slice.split_at_mut(components.component_size);
                let (state_bytes, term_bytes) = rest.split_at_mut(components.state_size);

                let comp_bytes: &[u8] = comp_bytes;
                let mut terminals = [0u32; 8];
                for i in 0..components.terminals {
                    let start = i * 4;
                    let end = start + 4;
                    terminals[i] = u32::from_ne_bytes(term_bytes[start..end].try_into().unwrap());
                }

                (components.post_stamp_fn)(
                    comp_bytes,
                    &mut self.numbers,
                    dt,
                    &terminals[..components.terminals],
                    state_bytes,
                );

                offset += total_size;
            }
        }
    }

    pub fn solve(&mut self) {
        self.numbers.solve();
    }
}
