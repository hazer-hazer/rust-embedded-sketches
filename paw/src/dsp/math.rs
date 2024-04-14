use micromath::F32Ext as _;

pub const PI2: f32 = core::f32::consts::PI * 2.0;

pub trait F32Ext: Sized + micromath::F32Ext {
    fn lerp(self, to: Self, f: Self) -> Self;
    fn remap_to_int_range(self, from: u32, to: u32) -> u32;
}

impl F32Ext for f32 {
    fn lerp(self, to: Self, f: Self) -> Self {
        self * (1.0 - f) + (to * f)
    }

    fn remap_to_int_range(self, from: u32, to: u32) -> u32 {
        ((self + 1.0) / 2.0 * (to - from) as f32) as u32 + from
    }
}

// Implement libm-only functions from num_complex but with micromath
pub trait Complex32Ext: Sized {
    fn from_polar(r: f32, theta: f32) -> Self;
    fn norm(self) -> f32;
}

impl Complex32Ext for num_complex::Complex32 {
    #[inline]
    fn from_polar(r: f32, theta: f32) -> Self {
        Self::new(r * theta.cos(), r * theta.sin())
    }

    #[inline]
    fn norm(self) -> f32 {
        self.re.hypot(self.re)
    }
}

pub trait SampleList<S = f32>: core::ops::Index<usize> {
    fn clipped_at(&self, index: usize) -> S;
    fn prev_clipped_at(&self, index: usize) -> S;
}

impl SampleList for [f32] {
    fn clipped_at(&self, index: usize) -> f32 {
        if index < self.len() {
            self[index]
        } else {
            0.0
        }
    }

    fn prev_clipped_at(&self, index: usize) -> f32 {
        if index > 0 {
            self.clipped_at(index - 1)
        } else {
            0.0
        }
    }
}

// Complex numbers //
macro_rules! complex_vec {
    ($(($re: expr, $im: expr)),*$(,)?) => (
        vec![
            $(num::complex::Complex::new($re, $im)),*
        ]
    );

    (($re: expr, $im: expr); $len: expr) => (
        vec![num::complex::Complex::new($re, $im); $len]
    );
}

pub(crate) use complex_vec;
