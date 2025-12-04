use std::{
    any::TypeId,
    collections::{HashMap, HashSet},
    mem::size_of,
};

use bytemuck::{
    bytes_of,
    checked::{try_from_bytes, try_from_bytes_mut},
};
use prettytable::{Cell, Row, Table};

use crate::{
    component::Component,
    net::{Net, c64},
    si::{format_complex_si_unitful, var_to_si_unit},
};

struct Components {
    buffer: Vec<u8>,
    stamp_fn: Box<dyn Fn(&[u8], &mut Net, f64, &[u32], &[u8])>,
    post_stamp_fn: Box<dyn Fn(&[u8], &Net, f64, &[u32], &mut [u8])>,
    describe_fn: Box<dyn Fn(&[u8], &Net, &[u32], &[u8]) -> Vec<(&'static str, c64)>>,
    component_size: usize,
    state_size: usize,
    priority: usize,
    terminals: usize,
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

    pub fn put<T: Component>(
        &mut self,
        component: T,
        terminals: [u32; T::TERMINAL_COUNT],
    ) -> &mut Self {
        let components = self.circuit.entry(TypeId::of::<T>()).or_insert(Components {
            buffer: vec![],
            stamp_fn: Box::new(
                |this: &[u8], net: &mut Net, dt: f64, ts: &[u32], state: &[u8]| {
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
                |this: &[u8], net: &Net, dt: f64, ts: &[u32], state: &mut [u8]| {
                    let this: &T = try_from_bytes(this).unwrap();
                    let state: &mut T::State = try_from_bytes_mut(state).unwrap();

                    let mut terminals = [0u32; T::TERMINAL_COUNT];
                    ts.iter().enumerate().for_each(|(i, t)| {
                        terminals[i] = *t;
                    });

                    this.post_stamp(net, dt, terminals, state);
                },
            ),
            describe_fn: Box::new(
                |this: &[u8], net: &Net, ts: &[u32], state: &[u8]| -> Vec<(&'static str, c64)> {
                    let this: &T = try_from_bytes(this).unwrap();
                    let state: &T::State = try_from_bytes(state).unwrap();

                    let mut terminals = [0u32; T::TERMINAL_COUNT];
                    ts.iter().enumerate().for_each(|(i, t)| {
                        terminals[i] = *t;
                    });

                    this.describe(net, terminals, state)
                },
            ),
            component_size: size_of::<T>(),
            state_size: size_of::<T::State>(),
            terminals: T::TERMINAL_COUNT,
            priority: T::PRIORITY,
        });

        components.buffer.extend(bytes_of(&component));
        components
            .buffer
            .extend_from_slice(bytes_of(&T::State::default()));

        (0..T::TERMINAL_COUNT).for_each(|i| {
            components.buffer.extend(bytes_of(&terminals[i]));
        });

        self
    }

    pub fn stamp(&mut self, net: &mut Net, dt: f64) {
        let mut values = self.circuit.iter_mut().collect::<Vec<_>>();
        values.sort_by_key(|(_, c)| c.priority);
        let values = values
            .into_iter()
            .map(|(c, _)| c)
            .copied()
            .collect::<Vec<_>>();

        // Ordinary stamping
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
                    net,
                    dt,
                    &terminals[..components.terminals],
                    state_bytes,
                );

                offset += total_size;
            }
        }

        // post stamp
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
                    net,
                    dt,
                    &terminals[..components.terminals],
                    state_bytes,
                );

                offset += total_size;
            }
        }
    }

    pub fn describe(&mut self, net: &Net) {
        let mut values = self.circuit.iter_mut().collect::<Vec<_>>();
        values.sort_by_key(|(_, c)| c.priority);
        let values = values
            .into_iter()
            .map(|(c, _)| c)
            .copied()
            .collect::<Vec<_>>();

        let mut table = Table::new();

        let mut headers = HashSet::<&'static str>::new();
        let mut rows = Vec::<HashMap<&'static str, c64>>::new();

        for i in values {
            let components = self.circuit.get_mut(&i).unwrap();

            let total_size =
                components.component_size + components.state_size + components.terminals * 4;

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

                let described = (components.describe_fn)(
                    comp_bytes,
                    net,
                    &terminals[..components.terminals],
                    state_bytes,
                );

                headers.extend(described.iter().map(|(k, _)| k));
                rows.push(described.into_iter().collect::<HashMap<_, _>>());

                offset += total_size;
            }
        }

        let mut headers = headers.iter().collect::<Vec<_>>();
        headers.sort();

        table.add_row(Row::new(
            headers
                .clone()
                .into_iter()
                .copied()
                .map(Cell::new)
                .collect::<Vec<_>>(),
        ));

        for row in rows {
            if row.is_empty() {
                continue;
            }

            let r = table.add_row(Row::empty());

            for &&h in &headers {
                let value = match row.get(h) {
                    Some(z) => format_complex_si_unitful(*z, var_to_si_unit(h).unwrap_or("")),
                    None => "".to_string(),
                };

                r.add_cell(Cell::new(&value));
            }
        }

        table.printstd();
    }
}
