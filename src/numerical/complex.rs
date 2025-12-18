use std::{
    fmt::{Debug, Display},
    ops::{Add, AddAssign, Div, Mul, MulAssign, Neg, Sub},
};

use bytemuck::{Pod, Zeroable};

use crate::si::format_complex_si;

#[derive(Clone, Copy, Pod, Zeroable, PartialEq, Default)]
#[repr(C)]
pub struct c64 {
    pub re: f64,
    pub im: f64,
}

impl c64 {
    pub const ZERO: Self = c64::new(0., 0.);
    pub const ONE: Self = c64::new(1., 0.);

    pub const fn new(re: f64, im: f64) -> Self {
        Self { im, re }
    }

    pub const fn real(re: f64) -> Self {
        Self { re, im: 0. }
    }

    pub const fn imag(im: f64) -> Self {
        Self { re: 0., im }
    }

    pub fn polar(amplitude: f64, angle_rad: f64) -> Self {
        Self {
            re: amplitude * angle_rad.cos(),
            im: amplitude * angle_rad.sin(),
        }
    }

    pub fn conj(self) -> Self {
        c64::new(self.re, -self.im)
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

impl MulAssign for c64 {
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
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

impl Display for c64 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format_complex_si(*self))
    }
}

impl Debug for c64 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format_complex_si(*self))
    }
}
