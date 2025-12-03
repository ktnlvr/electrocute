use nalgebra::{Complex, DMatrix, DVector};

pub type c64 = Complex<f64>;

pub struct Net {
    dim: usize,
    pub jacobian: DMatrix<c64>,
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

    pub fn reset(&mut self) {
        self.jacobian.fill(c64::ZERO);
        self.currents.fill(c64::ZERO);
    }

    pub fn clear_row_jacobian(&mut self, n: u32) {
        let n = n as usize;
        for i in 0..self.dim {
            self.jacobian[(n, i)] = c64::ZERO;
        }
        self.currents[n] = c64::ZERO;
    }

    pub fn add_jacobian(&mut self, a: u32, b: u32, value: c64) {
        self.jacobian[(a as usize, b as usize)] += value;
    }

    pub fn set_current(&mut self, n: u32, value: c64) {
        self.currents[n as usize] = value;
    }

    pub fn get_voltage_across(&self, from: u32, to: u32) -> c64 {
        self.voltages[to as usize] - self.voltages[from as usize]
    }

    pub fn get_current(&self, at: u32) -> c64 {
        self.currents[at as usize]
    }
}
