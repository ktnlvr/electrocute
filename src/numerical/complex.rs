use std::ops::{Add, AddAssign, Div, Mul, Neg, Sub};

use bytemuck::{Pod, Zeroable};

#[derive(Clone, Copy, Pod, Zeroable, PartialEq, Debug, Default)]
#[repr(C)]
pub struct c64 {
    pub im: f64,
    pub re: f64,
}

impl c64 {
    pub const ZERO: Self = c64::new(0., 0.);
    pub const ONE: Self = c64::new(1., 0.);

    pub const fn new(im: f64, re: f64) -> Self {
        Self { im, re }
    }

    pub fn polar(amplitude: f64, angle_rad: f64) -> Self {
        Self {
            re: amplitude * angle_rad.cos(),
            im: amplitude * angle_rad.sin(),
        }
    }

    pub fn norm(self) -> f64 {
        (self.re * self.re + self.im * self.im).sqrt()
    }

    pub fn arg(self) -> f64 {
        self.im.atan2(self.re)
    }
}

impl Add for c64 {
    type Output = c64;

    fn add(self, rhs: Self) -> Self::Output {
        c64::new(self.re + rhs.re, self.im + rhs.im)
    }
}

impl AddAssign for c64 {
    fn add_assign(&mut self, rhs: Self) {
        self.re += rhs.re;
        self.im += rhs.im;
    }
}

impl Sub for c64 {
    type Output = c64;

    fn sub(self, rhs: Self) -> Self::Output {
        c64::new(self.re - rhs.re, self.im - rhs.im)
    }
}

impl Mul for c64 {
    type Output = c64;

    fn mul(self, rhs: Self) -> Self::Output {
        c64::new(
            self.re * rhs.re - self.im * rhs.im,
            self.re * rhs.im + self.im * rhs.re,
        )
    }
}

impl Neg for c64 {
    type Output = c64;

    fn neg(self) -> Self::Output {
        c64::new(-self.re, -self.im)
    }
}

impl Div for c64 {
    type Output = c64;

    fn div(self, rhs: Self) -> Self::Output {
        let denom = rhs.re * rhs.re + rhs.im * rhs.im;

        c64::new(
            (self.re * rhs.re + self.im * rhs.im) / denom,
            (self.im * rhs.re - self.re * rhs.im) / denom,
        )
    }
}
