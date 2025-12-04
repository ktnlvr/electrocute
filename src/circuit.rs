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
    names: Vec<Option<String>>,
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

    pub fn put<T: Component>(
        &mut self,
        component: T,
        terminals: [u32; T::TERMINAL_COUNT],
    ) -> &mut Self {
        self.put_raw(component, terminals, None)
    }

    pub fn describe_component(&mut self, net: &Net, name: &str, value: &str) -> c64 {
        for components in self.circuit.values() {
            let total_size =
                components.component_size + components.state_size + components.terminals * 4;
            if total_size == 0 {
                continue;
            }

            let mut offset = 0;
            while offset + total_size <= components.buffer.len() {
                let (slice, _) = components.buffer[offset..].split_at(total_size);
                let (comp_bytes, rest) = slice.split_at(components.component_size);
                let (state_bytes, term_bytes) = rest.split_at(components.state_size);

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

                let comp_name = components.names[offset / total_size].as_deref();
                if comp_name == Some(name) {
                    for (k, v) in described {
                        if k == value {
                            return v;
                        }
                    }
                }

                offset += total_size;
            }
        }

        c64::new(0.0, 0.0)
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

    #[must_use]
    pub fn describe(
        &self,
        net: &Net,
    ) -> (Vec<String>, Vec<(Option<String>, HashMap<String, c64>)>) {
        let mut values = self.circuit.iter().collect::<Vec<_>>();
        values.sort_by_key(|(_, c)| c.priority);
        let values = values
            .into_iter()
            .map(|(c, _)| c)
            .copied()
            .collect::<Vec<_>>();

        let mut headers = HashSet::<&'static str>::new();
        let mut rows = Vec::<(Option<String>, HashMap<String, c64>)>::new();

        for i in values {
            let components = self.circuit.get(&i).unwrap();

            let total_size =
                components.component_size + components.state_size + components.terminals * 4;

            if total_size == 0 {
                continue;
            }

            let mut offset = 0;
            while offset + total_size <= components.buffer.len() {
                let (slice, _) = components.buffer[offset..].split_at(total_size);

                let (comp_bytes, rest) = slice.split_at(components.component_size);
                let (state_bytes, term_bytes) = rest.split_at(components.state_size);

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

                let name = components.names[offset / total_size].clone();

                headers.extend(described.iter().map(|(k, _)| k));
                rows.push((
                    name,
                    described
                        .into_iter()
                        .map(|(k, v)| (k.to_string(), v))
                        .collect::<HashMap<_, _>>(),
                ));

                offset += total_size;
            }
        }

        let mut headers = headers.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        headers.sort();

        (headers, rows)
    }
}
