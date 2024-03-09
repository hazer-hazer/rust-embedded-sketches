use crate::{
    audio::source::{AudioSource, AudioSourceProps},
    dsp::math::PI2,
};

use micromath::F32Ext as _;

pub struct SineSource {
    props: AudioSourceProps,
    freq: f32,
    sample_index: usize,
}

impl SineSource {
    pub fn new(sample_rate: u32, freq: f32, sample_index: usize) -> Self {
        Self {
            props: AudioSourceProps::new(1, sample_rate, None),
            freq,
            sample_index,
        }
    }
}

impl AudioSource for SineSource {
    fn props(&self) -> AudioSourceProps {
        self.props
    }
}

impl Iterator for SineSource {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        let sin = PI2 * self.freq * self.sample_index as f32 / self.sample_rate() as f32;

        self.sample_index = self.sample_index.wrapping_add(1);

        Some(sin.sin())
    }
}
