use crate::{
    audio::source::{AudioSource, AudioSourceProps},
    dsp::math::PI2,
};

use micromath::F32Ext as _;

pub struct SquareSource {
    props: AudioSourceProps,
    freq: f32,
    sample_index: usize,
}

impl SquareSource {
    pub fn new(sample_rate: u32, freq: f32) -> Self {
        Self {
            props: AudioSourceProps::infinite_mono(sample_rate),
            freq,
            sample_index: 0,
        }
    }
}

impl AudioSource for SquareSource {
    fn props(&self) -> AudioSourceProps {
        self.props
    }
}

impl Iterator for SquareSource {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        let sine = PI2 * self.freq * self.sample_index as f32 / self.sample_rate() as f32;

        self.sample_index = self.sample_index.wrapping_add(1);

        Some(sine.sin().signum())
    }
}
