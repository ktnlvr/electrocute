use std::{
    any::TypeId,
    collections::{HashMap, HashSet},
    f64::consts::PI,
};

use bytemuck::{bytes_of, try_from_bytes, try_from_bytes_mut};
use prettytable::{Cell, Row, Table};

use crate::{
    component::Component,
    net::{Net, c64},
};

struct Components {
    buffer: Vec<u8>,
    solve_fn: Box<dyn Fn(&[u8], &mut Net, f64, &[u32], &mut [u8])>,
    describe_fn: Box<dyn Fn(&[u8], &mut Net, &[u32], &mut [u8]) -> Vec<(&'static str, c64)>>,
    comp_size: usize,
    state_size: usize,
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

    pub fn put<T: 'static + Component>(
        &mut self,
        component: T,
        terminals: [u32; T::N],
    ) -> &mut Self {
        let type_id = TypeId::of::<T>();
        let slice = bytes_of(&component);

        let components = self.circuit.entry(type_id).or_insert(Components {
            buffer: vec![],
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
            describe_fn: Box::new(|this: &[u8], net: &mut Net, ts: &[u32], state: &mut [u8]| {
                let this: &T = try_from_bytes(this).unwrap();
                let mut terminals = [0u32; T::N];
                terminals
                    .iter_mut()
                    .zip(ts.iter())
                    .for_each(|(t, tt)| *t = *tt);
                let state: &mut T::State = try_from_bytes_mut(state).unwrap();

                this.describe(net, terminals, state)
            }),
            comp_size: std::mem::size_of::<T>(),
            state_size: if T::is_stateful() {
                std::mem::size_of::<T::State>()
            } else {
                0
            },
            terminals: T::N,
        });

        let buffer = &mut components.buffer;
        buffer.extend_from_slice(slice);
        if T::is_stateful() {
            buffer.extend_from_slice(bytes_of(&T::State::default()));
        }

        for terminal in terminals {
            buffer.extend_from_slice(bytes_of(&terminal));
        }

        self
    }

    pub fn fill_in_net(&mut self, net: &mut Net, dt: f64) {
        for comps in self.circuit.values_mut() {
            let comp_size = comps.comp_size;
            let state_size = comps.state_size;

            let n_terminals = comps.terminals;

            let terminal_bytes = n_terminals * std::mem::size_of::<u32>();
            let stride = comp_size + state_size + terminal_bytes;

            let buf_len = comps.buffer.len();
            let mut offset = 0;

            while offset + stride <= buf_len {
                let bytes = &mut comps.buffer[offset..offset + stride];

                let (comp_bytes, rest) = bytes.split_at_mut(comp_size);
                let (state_bytes, terminal_bytes) = rest.split_at_mut(state_size);

                let (_, terminals, _) = unsafe { terminal_bytes.align_to::<u32>() };

                (comps.solve_fn)(comp_bytes, net, dt, terminals, state_bytes);

                offset += stride;
            }
        }
    }

    pub fn describe(&mut self, net: &mut Net) {
        let mut headers = HashSet::<&'static str>::new();
        let mut rows = Vec::<HashMap<&'static str, c64>>::new();

        for comps in self.circuit.values_mut() {
            let comp_size = comps.comp_size;
            let state_size = comps.state_size;
            let n_terminals = comps.terminals;
            let terminal_bytes = n_terminals * std::mem::size_of::<u32>();
            let stride = comp_size + state_size + terminal_bytes;

            let buf_len = comps.buffer.len();
            let mut offset = 0;

            while offset + stride <= buf_len {
                let bytes = &mut comps.buffer[offset..offset + stride];

                let (comp_bytes, rest) = bytes.split_at_mut(comp_size);
                let (state_bytes, terminal_bytes) = rest.split_at_mut(state_size);

                let (_, terminals, _) = unsafe { terminal_bytes.align_to::<u32>() };

                let values = (comps.describe_fn)(comp_bytes, net, terminals, state_bytes);

                headers.extend(values.iter().map(|(h, _)| h));
                rows.push(values.into_iter().collect());

                offset += stride;
            }
        }

        let mut headers = headers.into_iter().collect::<Vec<_>>();
        headers.sort();

        let mut table = Table::new();
        table.add_row(Row::new(headers.iter().map(|h| Cell::new(h)).collect()));

        for row in rows {
            if row.is_empty() {
                continue;
            }

            let mut table_row: Vec<String> = vec![];

            for &h in &headers {
                table_row.push(match row.get(h) {
                    Some(value) => format!("{:.2}∠{:.2}°", value.norm(), value.arg() * 180.0 / PI),
                    None => "".to_string(),
                });
            }

            table.add_row(Row::new(table_row.iter().map(|s| Cell::new(s)).collect()));
        }

        table.printstd();
    }
}
