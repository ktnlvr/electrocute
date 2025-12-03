use nalgebra::{Complex, DMatrix, DVector};

pub type c64 = Complex<f64>;

pub struct Net {
    dim: usize,
    jacobian: DMatrix<c64>,
    pub voltages: DVector<c64>,
    pub currents: DVector<c64>,
}

impl Net {
    pub fn new(dim: usize) -> Self {
        Self {
            dim,
            jacobian: DMatrix::from_element(dim, dim, c64::ZERO),
            currents: DVector::from_element(dim, c64::ZERO),
            voltages: DVector::from_element(dim, c64::ZERO),
        }
    }

    pub fn solve(&mut self) {
        self.voltages = self.jacobian.clone().lu().solve(&self.currents).unwrap();
    }

    pub fn clear_row_jacobian(&mut self, n: u32) {
        for j in 0..self.dim {
            self.jacobian[(n as usize, j)] = c64::ZERO;
        }
    }

    pub fn add_jacobian(&mut self, a: u32, b: u32, value: c64) {
        self.jacobian[(a as usize, b as usize)] += value;
    }

    pub fn set_current(&mut self, n: u32, value: c64) {
        self.currents[n as usize] = value;
    }
}
