use core::marker::PhantomData;

use crate::{
    audio::source::{AudioSource, AudioSourceIter, AudioSourceProps},
    dsp::math::PI2,
};

use micromath::F32Ext as _;

#[derive(Clone, Copy)]
pub enum WaveForm {
    Sine,
    Square,
    Triangle,
    Saw,
    ReverseSaw,
    /// Pulse wave with duty cycle (0.0 - 1.0)
    Pulse(f32),
}

impl WaveForm {
    pub const VALUES: [Self; 6] = [
        Self::Sine,
        Self::Square,
        Self::Triangle,
        Self::Saw,
        Self::ReverseSaw,
        Self::Pulse(0.0),
    ];

    pub fn iter() -> impl Iterator<Item = WaveForm> {
        Self::VALUES.into_iter()
    }
}

pub struct SimpleFormSource<S = f32> {
    props: AudioSourceProps,
    // TODO: Maybe move to props
    current_channel: u16,
    /// Last computed value to repeat to all channels
    last_value: S,
    shape: WaveForm,
    freq: f32,
    sample_index: usize,
    _sample: PhantomData<S>,
}

impl SimpleFormSource {
    pub fn new(props: AudioSourceProps, shape: WaveForm, freq: f32) -> Self {
        if let WaveForm::Pulse(duty) = shape {
            assert!(duty >= 0.0 && duty <= 1.0);
        }

        Self {
            props,
            current_channel: 0,
            last_value: 0.0,
            shape,
            freq,
            sample_index: 0,
            _sample: PhantomData::default(),
        }
    }

    pub fn infinite_mono(sample_rate: u32, shape: WaveForm, freq: f32) -> Self {
        Self::new(AudioSourceProps::infinite_mono(sample_rate), shape, freq)
    }

    pub fn infinite_stereo(sample_rate: u32, shape: WaveForm, freq: f32) -> Self {
        Self::new(AudioSourceProps::infinite_stereo(sample_rate), shape, freq)
    }

    pub fn wave_len(&self) -> usize {
        (self.sample_rate() as f32 / self.freq) as usize
    }

    pub fn set_freq(&mut self, freq: f32) {
        self.freq = freq;
    }
}

impl AudioSource for SimpleFormSource {
    fn props(&self) -> AudioSourceProps {
        self.props
    }
}

// TODO: Add AudioSourceIter wrapper that will do:
// 1. Channels repetition (as I do below with `current_channel` and `last_value`)
// 2. `sample_index` logic
// Only `AudioSource` will implement iterator whereas `AudioSource`
// implementers will implement `AudioSourceIter`
impl AudioSourceIter for SimpleFormSource<f32> {
    type S = f32;

    fn next_sample(&mut self) -> Self::S {
        let s = if self.current_channel == 0 {
            let t = self.sample_index as f32;
            let sr = self.sample_rate() as f32;
            let period = sr / self.freq;
            let point = t / period; // t / p

            // Note: Sample index is for all channels
            self.sample_index = self.sample_index.wrapping_add(1);

            let s = match self.shape {
                WaveForm::Sine => (PI2 * point).sin(),
                // WaveForm::Square => (PI2 * point).sin().signum(),
                WaveForm::Square => {
                    if point.fract() < 0.5 {
                        1.0
                    } else {
                        -1.0
                    }
                }
                WaveForm::Triangle => 2.0 * (2.0 * (point - (point + 0.5).floor())).abs() - 1.0,
                WaveForm::Saw => 2.0 * (point - (point + 0.5).floor()),
                WaveForm::ReverseSaw => 2.0 * ((point + 0.5).floor() - point),
                WaveForm::Pulse(duty) => {
                    if (t % period / period) < duty {
                        1.0
                    } else {
                        -1.0
                    }
                }
            };

            self.last_value = s;

            s
        } else {
            self.last_value
        };

        self.current_channel = (self.current_channel + 1) % self.channels();

        s
    }
}

// impl Iterator for SimpleFormSource<i32> {
//     type Item = i32;

//     fn next(&mut self) -> Option<Self::Item> {
//         todo!()
//     }
// }

#[cfg(test)]
mod tests {
    use crate::audio::source::AudioSourceIter as _;

    use super::{SimpleFormSource, WaveForm};

    macro_rules! assert_sample {
        // ($sample: expr, $value: expr, $idx: expr) => {{
        //     let sample = $sample.unwrap();
        //     let diff = (sample - $value).abs();
        //     assert!(
        //         diff <= f32::EPSILON,
        //         "Must be {:?}, got {:?} at sample N {}; !(({:?} - {:?}).abs() ({:?}) <= f32::EPSILON)",
        //         $value,
        //         sample,
        //         $idx,
        //         sample,
        //         $value,
        //         diff,
        //     );
        // }};
        ($sample: expr, $value: expr, $idx: expr) => {{
            let sample = $sample;
            assert_eq!(
                sample, $value,
                "Must be {:?}, got {:?} at sample N {};",
                $value, sample, $idx,
            );
        }};
    }

    #[test]
    fn square() {
        let mut wave = SimpleFormSource::infinite_mono(100, WaveForm::Square, 5.0);

        for c in 0..5 {
            for i in 0..10 {
                assert_sample!(wave.next_sample(), 1.0, i + c);
            }

            for i in 10..20 {
                assert_sample!(wave.next_sample(), -1.0, i + c);
            }
        }
    }

    #[test]
    fn pulse() {
        let mut wave = SimpleFormSource::infinite_mono(100, WaveForm::Pulse(0.2), 5.0);

        for c in 0..5 {
            for i in 0..4 {
                assert_sample!(wave.next_sample(), 1.0, i + c);
            }

            for i in 4..20 {
                assert_sample!(wave.next_sample(), -1.0, i + c);
            }
        }
    }
}
