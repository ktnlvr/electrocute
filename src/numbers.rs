use nalgebra::{Complex, DMatrix, DVector};

pub type c64 = Complex<f64>;

pub struct Numbers {
    dim: usize,
    pub a: DMatrix<c64>,
    pub x: DVector<c64>,
    pub b: DVector<c64>,
}

impl Numbers {
    pub fn new(dim: usize) -> Self {
        // Ax = b
        Self {
            dim,
            a: DMatrix::from_element(dim, dim, c64::ZERO),
            b: DVector::from_element(dim, c64::ZERO),
            x: DVector::from_element(dim, c64::ZERO),
        }
    }

    pub fn solve(&mut self) {
        self.x = self.a.clone().lu().solve(&self.b).unwrap();
    }

    pub fn reset(&mut self) {
        self.a.fill(c64::ZERO);
        self.b.fill(c64::ZERO);
    }

    pub fn clear_row_jacobian(&mut self, j: u32) {
        let j = j as usize;
        for i in 0..self.dim {
            self.a[(j, i)] = c64::ZERO;
        }
        self.b[j] = c64::ZERO;
    }

    pub fn add_a(&mut self, i: u32, j: u32, value: c64) {
        self.a[(i as usize, j as usize)] += value;
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
