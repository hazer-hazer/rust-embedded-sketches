use core::marker::PhantomData;

use crate::{
    audio::source::{AudioSource, AudioSourceProps, Channels, Duration},
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
    /// Last computed value to repeat to all channels
    last_value: S,
    shape: WaveForm,
    freq: f32,
    _sample: PhantomData<S>,
}

impl SimpleFormSource {
    pub fn new(sample_rate: u32, shape: WaveForm, freq: f32) -> Self {
        if let WaveForm::Pulse(duty) = shape {
            assert!(duty >= 0.0 && duty <= 1.0);
        }

        Self {
            props: AudioSourceProps::new(
                Channels::mono(),
                sample_rate,
                Some(Duration::from_single_period(freq, sample_rate)),
            ),
            last_value: 0.0,
            shape,
            freq,
            _sample: PhantomData::default(),
        }
    }

    // pub fn infinite_mono(sample_rate: u32, shape: WaveForm, freq: f32) -> Self {
    //     Self::new(AudioSourceProps::infinite_mono(sample_rate), shape, freq)
    // }

    // pub fn infinite_stereo(sample_rate: u32, shape: WaveForm, freq: f32) -> Self {
    //     Self::new(AudioSourceProps::infinite_stereo(sample_rate), shape, freq)
    // }

    // pub fn wave_len(&self) -> usize {
    //     (self.sample_rate() as f32 / self.freq) as usize
    // }

    pub fn set_freq(&mut self, freq: f32) {
        self.freq = freq;
        self.set_duration(Duration::from_single_period(freq, self.sample_rate()));
        self.restart();
    }

    // pub fn is_finished(&self) -> bool {
    //     !self.looping && self.sample_index as f32 / (self.sample_rate() as f32 / self.freq) > 1.0
    // }
}

impl AudioSource for SimpleFormSource {
    #[inline]
    fn props(&self) -> AudioSourceProps {
        self.props
    }

    #[inline]
    fn props_mut(&mut self) -> &mut AudioSourceProps {
        &mut self.props
    }
}

// TODO: Add AudioSourceIter wrapper that will do:
// 1. Channels repetition (as I do below with `current_channel` and `last_value`)
// 2. `sample_index` logic
// Only `AudioSource` will implement iterator whereas `AudioSource`
// implementers will implement `AudioSourceIter`
impl Iterator for SimpleFormSource<f32> {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        let s = if self.next_channel() == 0 {
            if let Some(sample_index) = self.time_tick() {
                let t = sample_index as f32;
                let sr = self.sample_rate() as f32;
                let period = sr / self.freq;
                let point = t / period; // t / p

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

                Some(s)
            } else {
                None
            }
        } else {
            Some(self.last_value)
        };

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
        let mut wave = SimpleFormSource::new(100, WaveForm::Square, 5.0);

        for c in 0..5 {
            for i in 0..10 {
                assert_sample!(wave.next(), Some(1.0), i + c);
            }

            for i in 10..20 {
                assert_sample!(wave.next(), Some(-1.0), i + c);
            }
        }
    }

    #[test]
    fn pulse() {
        let mut wave = SimpleFormSource::new(100, WaveForm::Pulse(0.2), 5.0);

        for c in 0..5 {
            for i in 0..4 {
                assert_sample!(wave.next(), Some(1.0), i + c);
            }

            for i in 4..20 {
                assert_sample!(wave.next(), Some(-1.0), i + c);
            }
        }
    }
}
