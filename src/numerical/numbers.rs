use std::collections::{BTreeMap, HashMap};

use crate::numerical::{complex::c64, solve};

// CSR
pub struct LinearEquations {
    value_map: HashMap<(u32, u32), usize>,
    column_indices: Vec<u32>,
    row_pointers: Vec<u32>,
    a: Vec<c64>,
    x: Vec<c64>,
    b: Vec<c64>,
}

impl LinearEquations {
    pub fn from_coordinates(coordinates: impl IntoIterator<Item = (u32, u32)>) -> Self {
        let compressed = coordinates.into_iter().fold(
            BTreeMap::<u32, Vec<u32>>::default(),
            |mut acc, (i, j)| {
                acc.entry(i).or_insert_with(Vec::new).push(j);
                acc
            },
        );

        let mut value_map = HashMap::new();
        let mut row_pointers = Vec::new();
        let mut column_indices = Vec::new();
        let mut nnz = 0;

        let mut max_row: u32 = 0;
        let mut max_col: u32 = 0;

        for (i, mut js) in compressed {
            max_row = max_row.max(i);
            js.sort_unstable();

            row_pointers.push(nnz as u32);

            for j in js {
                max_col = max_col.max(j);
                value_map.insert((i, j), nnz as usize);
                column_indices.push(j);
                nnz += 1;
            }
        }

        row_pointers.push(nnz);

        let n_rows = (max_row + 1) as usize;
        let n_cols = (max_col + 1) as usize;

        LinearEquations {
            value_map,
            column_indices,
            row_pointers,
            a: vec![c64::ZERO; nnz as usize],
            x: vec![c64::ZERO; n_cols],
            b: vec![c64::ZERO; n_rows],
        }
    }

    fn dimensions(&self) -> (usize, usize) {
        (self.b.len(), self.x.len())
    }

    pub fn solve(&mut self) {
        let x = self.x.clone();
        self.x = solve(
            &self.a[..],
            &self.column_indices[..],
            &self.row_pointers,
            x,
            &self.b,
            100,
            1e-6,
        );
    }

    pub fn clear_row(&mut self, i: u32) {
        let row = i as usize;
        let start = self.row_pointers[row] as usize;
        let end = self.row_pointers[row + 1] as usize;

        for k in start..end {
            self.a[k] = c64::ZERO;
        }
    }

    pub fn add_a(&mut self, i: u32, j: u32, value: c64) {
        if let Some(&k) = self.value_map.get(&(i, j)) {
            self.a[k] += value;
        } else {
            panic!("Attempt to write to non-existent CSR entry ({}, {})", i, j);
        }
    }

    pub fn set_b(&mut self, i: u32, value: c64) {
        self.b[i as usize] = value;
    }

    pub fn add_b(&mut self, i: u32, value: c64) {
        self.b[i as usize] += value;
    }

    pub fn get_voltage_across(&self, from: u32, to: u32) -> c64 {
        self.x[from as usize] - self.x[to as usize]
    }

    pub fn get_current(&self, i: u32) -> c64 {
        self.b[i as usize]
    }
}

#[cfg(test)]
mod tests {
    use std::f64::EPSILON;

    use super::*;
    use crate::numerical::complex::c64;

    #[test]
    fn test_csr_construction() {
        let coords = vec![(0, 0), (0, 2), (1, 1), (2, 0), (2, 2)];
        let le = LinearEquations::from_coordinates(coords);

        let (rows, cols) = le.dimensions();
        assert_eq!(rows, 3);
        assert_eq!(cols, 3);

        assert_eq!(le.row_pointers.len(), 4);
        assert_eq!(le.column_indices, vec![0, 2, 1, 0, 2]);

        assert_eq!(le.value_map.get(&(0, 0)), Some(&0));
        assert_eq!(le.value_map.get(&(0, 2)), Some(&1));
        assert_eq!(le.value_map.get(&(1, 1)), Some(&2));
        assert_eq!(le.value_map.get(&(2, 0)), Some(&3));
        assert_eq!(le.value_map.get(&(2, 2)), Some(&4));
    }

    #[test]
    fn test_add_and_clear_a() {
        let coords = vec![(0, 0), (0, 1)];
        let mut le = LinearEquations::from_coordinates(coords);

        le.add_a(0, 0, c64::new(1.0, 0.0));
        le.add_a(0, 1, c64::new(2.0, 0.0));

        assert_eq!(le.a[0], c64::new(1.0, 0.0));
        assert_eq!(le.a[1], c64::new(2.0, 0.0));

        le.clear_row(0);
        assert_eq!(le.a[0], c64::ZERO);
        assert_eq!(le.a[1], c64::ZERO);
    }

    #[test]
    fn test_set_and_add_b() {
        let coords = vec![(0, 0), (1, 1)];
        let mut le = LinearEquations::from_coordinates(coords);

        le.set_b(0, c64::new(1.0, 1.0));
        le.add_b(0, c64::new(2.0, -1.0));
        le.set_b(1, c64::new(0.5, 0.5));

        assert_eq!(le.b[0], c64::new(3.0, 0.0));
        assert_eq!(le.b[1], c64::new(0.5, 0.5));
    }

    #[test]
    fn test_get_voltage_across() {
        let coords = vec![(0, 0), (1, 1)];
        let mut le = LinearEquations::from_coordinates(coords);

        le.x[0] = c64::new(5.0, 0.0);
        le.x[1] = c64::new(2.0, 0.0);

        let v = le.get_voltage_across(0, 1);
        assert_eq!(v, c64::new(3.0, 0.0));
    }

    #[test]
    fn test_get_current() {
        let coords = vec![(0, 0)];
        let mut le = LinearEquations::from_coordinates(coords);

        le.set_b(0, c64::new(4.0, 0.0));
        let i = le.get_current(0);
        assert_eq!(i, c64::new(4.0, 0.0));
    }

    #[test]
    #[should_panic]
    fn test_add_a_invalid_index() {
        let coords = vec![(0, 0)];
        let mut le = LinearEquations::from_coordinates(coords);
        le.add_a(0, 1, c64::new(1.0, 0.0));
    }

    #[test]
    fn test_solve() {
        let mut le = LinearEquations::from_coordinates(vec![(0, 0), (1, 1)]);
        le.add_a(0, 0, c64::new(2.0, 0.0));
        le.add_a(1, 1, c64::new(3.0, 0.0));
        le.set_b(0, c64::new(4.0, 0.0));
        le.set_b(1, c64::new(9.0, 0.0));
        le.solve();
        assert!(le.x[0].re - 2.0 < EPSILON);
        assert!(le.x[1].re - 3.0 < EPSILON);
    }
}
