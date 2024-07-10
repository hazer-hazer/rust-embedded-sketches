pub mod buffered;
pub mod looped;
mod mix;
pub mod restarting;
pub mod zero;

use core::iter::Iterator;

use crate::dsp::sample::Sample;

#[derive(Clone, Copy)]
pub struct Channels {
    pub count: u8,
    pub current: u8,
}

impl Channels {
    pub fn new(count: u8, channel: u8) -> Self {
        assert!(count > 0 && channel < count);
        Self {
            count,
            current: channel,
        }
    }

    pub fn stereo(start_channel: u8) -> Self {
        Self::new(2, start_channel)
    }

    pub fn stereo_first() -> Self {
        Self::new(2, 0)
    }

    pub fn mono() -> Self {
        Self::new(1, 0)
    }

    fn restart(&mut self) {
        self.current = 0;
    }

    fn next_channel(&mut self) -> u8 {
        let last = self.current;
        self.current = (self.current + 1) % self.count;
        last
    }
}

#[derive(Clone, Copy)]
pub struct Duration {
    pub real: core::time::Duration,
    pub samples: u32,
}

impl Duration {
    pub fn from_real(real: core::time::Duration, sample_rate: u32) -> Self {
        Self {
            real,
            samples: (real.as_secs_f32() * sample_rate as f32) as u32,
        }
    }

    pub fn from_samples(samples: u32, sample_rate: u32) -> Self {
        Self {
            real: core::time::Duration::from_secs_f32(samples as f32 / sample_rate as f32),
            samples,
        }
    }

    pub fn from_single_period(freq: f32, sample_rate: u32) -> Self {
        Self::from_samples((sample_rate as f32 / freq) as u32, sample_rate)
    }
}

#[derive(Clone, Copy)]
pub struct AudioSourceTime {
    total: Option<Duration>,
    sample_index: u32,
}

impl AudioSourceTime {
    pub fn new(total: Option<Duration>) -> Self {
        Self {
            total,
            sample_index: 0,
        }
    }

    pub fn infinite() -> Self {
        Self {
            total: None,
            sample_index: 0,
        }
    }

    fn restart(&mut self) {
        self.sample_index = 0;
    }

    fn set_duration(&mut self, duration: Duration) {
        self.total = Some(duration);
        self.restart()
    }

    fn tick(&mut self) -> Option<u32> {
        let point = self.sample_index;

        self.sample_index += 1;

        if let Some(total) = self.total {
            if self.sample_index <= total.samples {
                Some(point)
            } else {
                None
            }
        } else {
            Some(point)
        }
    }

    fn is_finished(&self) -> bool {
        self.total
            .is_some_and(|dur| self.sample_index >= dur.samples)
    }
}

#[derive(Clone, Copy)]
pub struct AudioSourceProps {
    channels: Channels,
    sample_rate: u32,
    time: AudioSourceTime,
}

impl AudioSourceProps {
    pub fn new(channels: Channels, sample_rate: u32, duration: Option<Duration>) -> Self {
        Self {
            channels,
            sample_rate,
            time: AudioSourceTime::new(duration),
        }
    }

    // pub fn infinite_mono(sample_rate: u32) -> Self {
    //     Self::new(1, sample_rate, None)
    // }

    // pub fn infinite_stereo(sample_rate: u32) -> Self {
    //     Self::new(2, sample_rate, None)
    // }

    // pub fn set_duration_samples(&mut self, samples: u32) {
    //     self.time
    //         .set_duration(Duration::from_samples(samples, self.sample_rate))
    // }

    // pub fn set_duration_single_cycle(&mut self, freq: f32) {
    //     self.set_duration_samples((self.sample_rate as f32 / freq) as u32)
    // }
}

// pub trait Sample: Clone + Copy {
//     fn lerp(self, to: Self, f: f32) -> Self;
//     fn zero() -> Self;
//     fn saturating_add(self, rhs: Self) -> Self;
//     fn amplify(self, amount: f32) -> Self;
//     fn remap_int_range(self, from: i32, to: i32) -> i32;
//     fn remap_uint_range(self, from: u32, to: u32) -> u32;
// }

// impl Sample for f32 {
//     // fn lerp(self, to: Self, f: f32) -> Self {
//     //     self * (1.0 - f) + (to * f)
//     // }

//     // fn zero() -> Self {
//     //     0f32
//     // }

//     // fn saturating_add(self, rhs: Self) -> Self {
//     //     let sum = self + rhs;
//     //     if sum < -1.0 {
//     //         -1.0
//     //     } else if sum > 1.0 {
//     //         1.0
//     //     } else {
//     //         sum
//     //     }
//     // }

//     // fn amplify(self, amount: f32) -> Self {
//     //     self * amount
//     // }

//     // fn remap_int_range(self, from: i32, to: i32) -> i32 {
//     //     assert!(from <= to);
//     //     (self * (to - from) as f32) as i32 + from
//     // }

//     // fn remap_uint_range(self, from: u32, to: u32) -> u32 {
//     //     assert!(from <= to);
//     //     ((self + 1.0) * (to - from) as f32 / 2.0).round() as u32 + from
//     // }
// }

// TODO: Rewrite sound source to frames of stereo samples
pub trait AudioSource: Iterator
where
    Self::Item: Sample,
    Self: Sized,
{
    // !!! #[inline]
    fn props(&self) -> AudioSourceProps;
    // !!! #[inline]
    fn props_mut(&mut self) -> &mut AudioSourceProps;

    #[inline]
    fn channels_count(&self) -> u8 {
        self.props().channels.count
    }

    #[inline]
    fn current_channel(&self) -> u8 {
        self.props().channels.current
    }

    #[inline]
    fn sample_rate(&self) -> u32 {
        self.props().sample_rate
    }

    #[inline]
    fn duration(&self) -> Option<core::time::Duration> {
        self.props().time.total.map(|duration| duration.real)
    }

    #[inline]
    fn duration_samples(&self) -> Option<u32> {
        self.props().time.total.map(|duration| duration.samples)
    }

    #[inline]
    fn is_finished(&self) -> bool {
        self.props().time.is_finished()
    }

    #[inline]
    fn next_channel(&mut self) -> u8 {
        self.props().channels.next_channel()
    }

    #[inline]
    fn time_tick(&mut self) -> Option<u32> {
        self.props().time.tick()
    }

    #[inline]
    fn set_duration(&mut self, duration: Duration) {
        self.props_mut().time.set_duration(duration)
    }

    #[inline]
    fn restart(&mut self) {
        self.props_mut().time.restart();
        self.props_mut().channels.restart();
    }

    // Builders //
    #[inline]
    fn channels(mut self, channels: u8) -> Self {
        self.props_mut().channels.count = channels;
        self
    }

    #[inline]
    fn mono(self) -> Self {
        self.channels(1)
    }

    #[inline]
    fn stereo(self) -> Self {
        self.channels(2)
    }

    // fn samples_left(&self) -> Option<u64> {
    //     self.props().duration.map(|d| d.samples_left)
    // }

    // fn duration_left(&self) -> Option<Duration> {
    //     self.props()
    //         .duration
    //         .map(|d| Duration::from_nanos(1_000_000_000 / d.samples_left))
    // }

    // Iterators //
    // fn mix<O: AudioSource>(self, other: O) -> Mix<Self, O>
    // where
    //     O::Item: Sample,
    // {
    //     mix(self, other)
    // }
}

pub trait Pausable {
    fn playing(&self) -> bool;
    fn pause(&mut self);
}

pub trait Stoppable {
    /// Stop not always mean real "stopped" state set,
    /// it may mean that we are at the start of the sample
    /// list and next iteration will give None
    fn is_stopped(&self) -> bool;
    fn stop(&mut self) -> bool;
}
