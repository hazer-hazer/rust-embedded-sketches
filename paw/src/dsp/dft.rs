use alloc::vec::Vec;
use micromath::F32Ext as _;
use num::{complex::Complex32, Complex, Zero};

use super::math::{complex_vec, PI2};

#[derive(Clone, Copy)]
pub enum DftDirection {
    Forward,
    Inverse,
}

pub trait Dft<S = f32> {
    fn len(&self) -> usize;
    fn direction(&self) -> DftDirection;

    fn process(&mut self, signal: &[S], spectrum_len: usize) -> Vec<Complex32>;
}

pub struct NaiveDft {
    // Note: May be rewritten to const generic array usage
    twiddles: Vec<Complex32>,
    direction: DftDirection,
}

fn make_twiddle(index: usize, size: usize, direction: DftDirection) -> Complex32 {
    let angle = -PI2 / size as f32 * index as f32;
    let twiddle = Complex {
        re: angle.cos(),
        im: angle.sin(),
    };

    match direction {
        DftDirection::Forward => twiddle,
        DftDirection::Inverse => twiddle.conj(),
    }
}

impl NaiveDft {
    fn new(size: usize, direction: DftDirection) -> Self {
        Self {
            twiddles: (0..size)
                .map(|i| make_twiddle(i, size, direction))
                .collect(),
            direction,
        }
    }
}

impl Dft for NaiveDft {
    fn direction(&self) -> DftDirection {
        self.direction
    }

    fn len(&self) -> usize {
        self.twiddles.len()
    }

    fn process(&mut self, signal: &[f32], spectrum_len: usize) -> Vec<Complex32> {
        let mut spectrum = complex_vec![(0., 0.); spectrum_len];

        for (k, freq) in spectrum.iter_mut().enumerate() {
            *freq = Zero::zero();

            let mut twiddle_index = 0;

            for sample in signal {
                *freq += self.twiddles[twiddle_index] * sample;

                twiddle_index += k;
                if twiddle_index >= self.twiddles.len() {
                    twiddle_index -= self.twiddles.len();
                }
            }
        }

        spectrum
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec::Vec;
    use num::{Complex, Zero};
    use num_complex::Complex32;

    use crate::dsp::math::{complex_vec, Complex32Ext as _};

    use super::{Dft, DftDirection, NaiveDft};

    fn reference_dft(signal: &[Complex32], spectrum_len: usize) -> Vec<Complex32> {
        let mut spectrum = complex_vec![(0., 0.); spectrum_len];
        for (k, spec_bin) in spectrum.iter_mut().enumerate() {
            let mut sum = Zero::zero();
            for (i, sample) in signal.iter().enumerate() {
                let angle = -1. * (i * k) as f32 * 2. * core::f32::consts::PI / signal.len() as f32;
                let twiddle = Complex32::from_polar(1., angle);

                sum += twiddle * sample;
            }
            *spec_bin = sum;
        }
        spectrum
    }

    fn test_dft(signal: &[Complex32], expected: &[Complex32]) {
        assert_eq!(signal.len(), expected.len());

        let reference = reference_dft(signal, expected.len());

        assert_eq!(reference.len(), expected.len());
        let error = reference
            .iter()
            .zip(expected.iter())
            .fold(0., |error, (a, b)| error + (a - b).norm());

        const ALLOWED_ERROR: f32 = 0.1;

        let avg_error = error / reference.len() as f32;
        assert!(
            avg_error < ALLOWED_ERROR,
            "Average error of {avg_error} on tested DFT is more than allowed {ALLOWED_ERROR}"
        );
    }

    #[test]
    fn zero_dft() {
        let mut zero_dft = NaiveDft::new(0, DftDirection::Forward);
        zero_dft.process(&[], 0);
    }

    #[test]
    fn known_2_samples() {
        let signal = complex_vec![(1., 0.), (-1., 0.)];
        let spectrum = complex_vec![(0., 0.), (2., 0.)];
        test_dft(&signal, &spectrum);
    }

    #[test]
    fn known_3_samples() {
        let signal = complex_vec![(1., 1.), (2., -3.), (-1., 4.)];
        let spectrum = complex_vec![(2., 2.), (-5.562177, -2.098076), (6.562178, 3.09807)];
        test_dft(&signal, &spectrum);
    }

    #[test]
    fn known_4_samples() {
        let signal = complex_vec![(0., 1.), (2.5, -3.), (-1., -1.), (4., 0.)];
        let spectrum = complex_vec![(5.5, -3.), (-2., 3.5), (-7.5, 3.), (4., 0.5)];
        test_dft(&signal, &spectrum);
    }

    #[test]
    fn known_5_samples() {
        let signal = complex_vec![(1., 1.), (2., 2.), (3., 3.), (4., 4.), (5., 5.), (6., 6.)];
        let spectrum = complex_vec![
            (21., 21.),
            (-8.16, 2.16),
            (-4.76, -1.24),
            (-3., -3.),
            (-1.24, -4.76),
            (2.16, -8.16)
        ];
        test_dft(&signal, &spectrum);
    }
}
