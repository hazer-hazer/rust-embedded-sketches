use crate::dsp::math::PI2;
use micromath::F32Ext as _;

const WAVE_TABLE_SIZE: usize = 128;

pub type WaveTableGenFn = fn(point: f32) -> f32;

pub struct WaveTable {
    samples: [f32; WAVE_TABLE_SIZE],
}

impl core::ops::Index<usize> for WaveTable {
    type Output = f32;

    fn index(&self, index: usize) -> &Self::Output {
        &self.samples[index]
    }
}

impl WaveTable {
    // Constructors //
    pub fn gen(gen: WaveTableGenFn) -> Self {
        let mut samples = [0.0; WAVE_TABLE_SIZE];
        for (n, s) in samples.iter_mut().enumerate() {
            *s = gen(n as f32 / WAVE_TABLE_SIZE as f32);
        }
        Self { samples }
    }

    pub fn sine() -> Self {
        Self::gen(|point| (PI2 * point).sin())
    }

    pub fn square() -> Self {
        Self::gen(|point| (PI2 * point).sin().signum())
    }

    pub fn fast_square() -> Self {
        Self::gen(|point| 1.0 * if point < 0.5 { -1. } else { 1. })
    }

    pub fn triangle() -> Self {
        Self::gen(|point| 2.0 * (2.0 * (point - (point + 0.5).floor())).abs() - 1.0)
    }

    pub fn saw() -> Self {
        Self::gen(|point| 2.0 * (point - (point + 0.5).floor()))
    }

    pub fn rev_saw() -> Self {
        Self::gen(|point| 2.0 * ((point + 0.5).floor() - point))
    }

    //
    pub fn len(&self) -> usize {
        self.samples.len()
    }
}
