mod mix;
pub mod zero;

use core::{iter::Iterator, marker::PhantomData, time::Duration};

use self::mix::{mix, Mix};

#[derive(Clone, Copy)]
pub struct AudioSourceDuration {
    total: Duration,
    samples_left: u64,
}

impl AudioSourceDuration {
    pub fn new(total: Duration) -> Self {
        Self {
            total,
            samples_left: 0,
        }
    }

    pub fn tick(&mut self) -> bool {
        if self.samples_left == 0 {
            return false;
        }
        self.samples_left -= 1;
        true
    }
}

pub trait Sample: Clone + Copy {
    fn lerp(self, to: Self, f: f32) -> Self;
    fn zero() -> Self;
    fn saturating_add(self, rhs: Self) -> Self;
    fn amplify(self, amount: f32) -> Self;
    fn remap_int_range(self, from: i32, to: i32) -> i32;
    fn remap_uint_range(self, from: u32, to: u32) -> u32;
}

impl Sample for f32 {
    fn lerp(self, to: Self, f: f32) -> Self {
        self * (1.0 - f) + (to * f)
    }

    fn zero() -> Self {
        0f32
    }

    fn saturating_add(self, rhs: Self) -> Self {
        let sum = self + rhs;
        if sum < -1.0 {
            -1.0
        } else if sum > 1.0 {
            1.0
        } else {
            sum
        }
    }

    fn amplify(self, amount: f32) -> Self {
        self * amount
    }

    fn remap_int_range(self, from: i32, to: i32) -> i32 {
        assert!(from <= to);
        (self * (to - from) as f32) as i32 + from
    }

    fn remap_uint_range(self, from: u32, to: u32) -> u32 {
        assert!(from <= to);
        ((self + 1.0) * (to - from) as f32 / 2.0).round() as u32 + from
    }
}

pub trait AudioSource: Iterator
where
    Self::Item: Sample,
{
    fn props(&self) -> AudioSourceProps;

    fn channels(&self) -> u16 {
        self.props().channels
    }

    fn sample_rate(&self) -> u32 {
        self.props().sample_rate
    }

    fn total_duration(&self) -> Option<Duration> {
        self.props().duration.map(|d| d.total)
    }

    fn samples_left(&self) -> Option<u64> {
        self.props().duration.map(|d| d.samples_left)
    }

    fn duration_left(&self) -> Option<Duration> {
        self.props()
            .duration
            .map(|d| Duration::from_nanos(1_000_000_000 / d.samples_left))
    }

    // Iterators //
    // fn mix<O: AudioSource>(self, other: O) -> Mix<Self, O>
    // where
    //     O::Item: Sample,
    // {
    //     mix(self, other)
    // }
}

#[derive(Clone, Copy)]
pub struct AudioSourceProps {
    pub channels: u16,
    pub sample_rate: u32,
    pub duration: Option<AudioSourceDuration>,
}

impl AudioSourceProps {
    pub fn new(channels: u16, sample_rate: u32, duration: Option<Duration>) -> Self {
        Self {
            channels,
            sample_rate,
            duration: duration.map(|total| AudioSourceDuration::new(total)),
        }
    }

    pub fn infinite_mono(sample_rate: u32) -> Self {
        Self::new(1, sample_rate, None)
    }
}
