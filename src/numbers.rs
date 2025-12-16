use std::{collections::HashMap, mem::replace};

use nalgebra::Complex;

pub type c64 = Complex<f64>;

pub struct CSR {
    map: HashMap<(u32, u32), usize>,
    column_indices: Vec<u32>,
    row_pointers: Vec<u32>,
    a: Vec<c64>,
    x: Vec<c64>,
    b: Vec<c64>,
}

pub struct COO {
    dimensions: usize,
    data: Vec<c64>,
}

pub enum Numbers {
    Coordinates(COO),
    Compressed(CSR),
}

pub const NULL_COMPLEX_NUMBERS: c64 = c64::new(f64::NEG_INFINITY, f64::INFINITY);

impl Numbers {
    pub fn new(dimensions: usize) -> Self {
        Self::Coordinates(COO {
            dimensions,
            data: vec![NULL_COMPLEX_NUMBERS; dimensions * dimensions],
        })
    }

    fn bake(&mut self) -> &mut CSR {
        if let Numbers::Compressed(baked) = self {
            return baked;
        }

        let Numbers::Coordinates(coords) = replace(
            self,
            Numbers::Coordinates(COO {
                dimensions: 0,
                data: Vec::new(),
            }),
        ) else {
            unreachable!();
        };

        let n = coords.dimensions;
        let mut map = HashMap::new();
        let mut row_indices = Vec::new();
        let mut column_indices = Vec::new();
        let mut a = Vec::new();

        for i in 0..n {
            for j in 0..n {
                let idx = i * n + j;
                let value = coords.data[idx];

                if value != NULL_COMPLEX_NUMBERS {
                    let k = a.len();
                    map.insert((i as u32, j as u32), k);
                    row_indices.push(i as u32);
                    column_indices.push(j as u32);
                    a.push(value);
                }
            }
        }

        let baked = CSR {
            map,
            row_pointers: row_indices,
            column_indices,
            a,
            x: vec![c64::ZERO; n],
            b: vec![c64::ZERO; n],
        };

        *self = Numbers::Compressed(baked);

        match self {
            Numbers::Compressed(baked) => baked,
            _ => unreachable!(),
        }
    }

    fn dimensions(&self) -> usize {
        match self {
            Numbers::Coordinates(coordinates) => coordinates.dimensions,
            Numbers::Compressed(numbers) => numbers.column_indices.len(),
        }
    }

    pub fn solve(&mut self) {
        todo!();
    }

    pub fn reset(&mut self) {
        let baked = self.bake();
        baked.a.clear();
        baked.b.clear();
    }

    pub fn clear_row_jacobian(&mut self, j: u32) {
        let CSR {
            row_pointers: row_indices,
            a,
            ..
        } = self.bake();
        for (&row_j, v) in row_indices.iter().zip(a) {
            if row_j == j {
                *v = c64::ZERO;
            }
        }
    }

    pub fn add_a(&mut self, i: u32, j: u32, value: c64) {
        let baked = self.bake();
        let idx = baked.map[&(i, j)];
        baked.a[idx] += value;
    }

    pub fn set_b(&mut self, i: u32, value: c64) {
        let baked = self.bake();
        baked.b[i as usize] = value;
    }

    pub fn add_b(&mut self, i: u32, value: c64) {
        let baked = self.bake();
        baked.b[i as usize] += value;
    }

    pub fn get_voltage_across(&self, from: u32, to: u32) -> c64 {
        match self {
            Numbers::Coordinates(_) => c64::ZERO,
            Numbers::Compressed(CSR { x, .. }) => x[from as usize] - x[to as usize],
        }
    }

    pub fn get_current(&self, i: u32) -> c64 {
        match self {
            Numbers::Coordinates(_) => c64::ZERO,
            Numbers::Compressed(CSR { b, .. }) => b[i as usize],
        }
    }
}
