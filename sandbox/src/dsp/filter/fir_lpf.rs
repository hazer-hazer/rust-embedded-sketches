use crate::dsp::math::PI2;

use super::filter::Filter;
use alloc::vec::{self, Vec};
use micromath::F32Ext;

// TODO: Rewrite when generic const calculations will be implemented.
// https://github.com/rust-lang/rust/issues/76560

pub struct FirLpf {
    sample_rate: u32,
    cutoff_freq: f32,

    // TODO: Must be Order + 1
    filters: Vec<f32>,
    z_pointer: usize,
    // TODO: Must be Order - 2 size, but cannot do math on Order as it's a generic param
    z_buf: Vec<f32>,
}

impl FirLpf {
    pub fn new(order: u8, sample_rate: u32, cutoff_freq: f32) -> Self {
        let omega = PI2 * cutoff_freq / sample_rate as f32;

        let mut filters = vec![0.0; order as usize + 1];

        let mut dc = 0f32;

        for index in 0..filters.len() {
            let order_offset = index as f32 - order as f32 / 2.0;

            if order_offset.floor() == 0.0 {
                // Middle point
                filters[index] = omega;
            } else {
                filters[index] = (omega * order_offset).sin() / order_offset;
                filters[index] *= 0.54 - 0.46 * (PI2 * index as f32 / order as f32).cos();
            }

            dc += filters[index];
        }

        for i in 0..filters.len() {
            filters[i] /= dc;
        }

        Self {
            sample_rate,
            cutoff_freq,
            filters,
            z_pointer: 0,
            z_buf: vec![0.0; order as usize - 2],
        }
    }
}

impl Filter for FirLpf {
    fn reset(&mut self) {
        self.z_buf = vec![0.0; self.filters.len() - 1];
        self.z_pointer = 0;
    }

    fn filter(&mut self, sample: f32) -> f32 {
        self.z_buf[self.z_pointer] = sample;

        let mut output = 0.0;
        for i in 0..self.z_buf.len() {
            output += self.filters[i] * self.z_buf[(self.z_pointer + i) % self.z_buf.len()];
        }

        // let output = self
        //     .filters
        //     .iter()
        //     .enumerate()
        //     .fold(0f32, |output, (i, flt)| {
        //         output + flt * self.z_buf[(self.z_pointer + i) % self.z_buf.len()].as_()
        //     });
        self.z_pointer = (self.z_pointer + 1) % self.z_buf.len();

        output
    }
}
