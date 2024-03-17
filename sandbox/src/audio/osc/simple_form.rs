use core::marker::PhantomData;

use crate::{
    audio::source::{AudioSource, AudioSourceProps},
    dsp::math::PI2,
};

use micromath::F32Ext as _;

#[derive(defmt::Format, Clone, Copy)]
pub enum WaveForm {
    Sine,
    Square,
    Triangle,
    Saw,
    ReverseSaw,
    // /// Pulse wave with duty cycle (0.0 - 1.0)
    // Pulse(f32),
}

impl WaveForm {
    pub const VALUES: [Self; 5] = [
        Self::Sine,
        Self::Square,
        Self::Triangle,
        Self::Saw,
        Self::ReverseSaw,
    ];

    pub fn iter() -> impl Iterator<Item = WaveForm> {
        Self::VALUES.into_iter()
    }
}

pub struct SimpleFormSource<S = f32> {
    props: AudioSourceProps,
    shape: WaveForm,
    freq: f32,
    sample_index: usize,
    _sample: PhantomData<S>,
}

impl SimpleFormSource {
    pub fn new(props: AudioSourceProps, shape: WaveForm, freq: f32) -> Self {
        Self {
            props,
            shape,
            freq,
            sample_index: 0,
            _sample: PhantomData::default(),
        }
    }

    pub fn infinite_mono(sample_rate: u32, shape: WaveForm, freq: f32) -> Self {
        Self::new(AudioSourceProps::infinite_mono(sample_rate), shape, freq)
    }

    pub fn wave_len(&self) -> usize {
        (self.sample_rate() as f32 / self.freq) as usize
    }
}

impl AudioSource for SimpleFormSource {
    fn props(&self) -> AudioSourceProps {
        self.props
    }
}

impl Iterator for SimpleFormSource<f32> {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        let t = self.sample_index as f32;
        let sr = self.sample_rate() as f32;
        let period = sr / self.freq;
        let point = t / period; // t / p

        let s = match self.shape {
            WaveForm::Sine => (PI2 * point).sin(),
            WaveForm::Square => (PI2 * point).sin().signum(),
            WaveForm::Triangle => 2.0 * (2.0 * (point - (point + 0.5).floor())).abs() - 1.0,
            WaveForm::Saw => 2.0 * (point - (point + 0.5).floor()),
            WaveForm::ReverseSaw => 2.0 * ((point + 0.5).floor() - point),
            // WaveForm::Pulse(duty) => ,
        };

        self.sample_index = self.sample_index.wrapping_add(1);

        Some(s)
    }
}

impl Iterator for SimpleFormSource<i32> {
    type Item = i32;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}
