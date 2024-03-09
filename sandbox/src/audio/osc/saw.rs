use crate::audio::source::{AudioSource, AudioSourceProps};

use micromath::F32Ext as _;

pub struct SawSource {
    props: AudioSourceProps,
    freq: f32,
    sample_index: usize,
}

impl SawSource {
    pub fn new(sample_rate: u32, freq: f32) -> Self {
        Self {
            props: AudioSourceProps::infinite_mono(sample_rate),
            freq,
            sample_index: 0,
        }
    }
}

impl AudioSource for SawSource {
    fn props(&self) -> AudioSourceProps {
        self.props
    }
}

impl Iterator for SawSource {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        let wave_len = self.sample_rate() as f32 / self.freq;
        let x = self.sample_index as f32;
        let saw = 2.0 * (x as f32 / wave_len - (x / wave_len + 0.5).floor());

        self.sample_index = self.sample_index.wrapping_add(1);

        Some(saw)
    }
}
