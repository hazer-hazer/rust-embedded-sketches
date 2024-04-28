use alloc::vec::Vec;

use super::{
    filter::{filter::Filter, fir_lpf::FirLpf},
    interpolator::{InterpolationMode, Interpolator},
};

use micromath::F32Ext as _;

#[derive(Clone, Copy)]
pub enum LpfKind {
    // IIR,
    FIR,
}

// TODO: Different LPFs and settings
// const fn defaultLpfOrder(lpf_kind: LpfKind) -> usize {
//     match lpf_kind {
//         LpfKind::FIR => 71,
//     }
// }

const DEFAULT_INTERPOLATION_MODE: InterpolationMode = InterpolationMode::Cubic;
const DEFAULT_TENSION: f32 = 0.0;
const DEFAULT_SINC_FILTER_SIZE: usize = 6;
const DEFAULT_LPF_ORDER: u8 = 71;

pub struct Resampler {
    from_rate: u32,
    to_rate: u32,
    mode: InterpolationMode,
    tension: f32,
    sinc_filter_size: usize,
    lpf: Option<FirLpf>,
}

impl Resampler {
    // TODO: Possible can avoid heap (Vec) when const generic math implemented,
    // so we can calculate rate thus determine the result size
    pub fn resample(&mut self, samples: &[f32]) -> Vec<f32> {
        let rate = (self.to_rate as f32 - self.from_rate as f32) / self.from_rate as f32 + 1.0;
        let from_len = samples.len();
        let to_len = ((from_len as f32 * rate).ceil() as usize).max(self.to_rate as usize);

        let interpolator = Interpolator::new(
            from_len,
            to_len,
            self.mode,
            Some(self.tension),
            self.sinc_filter_size,
        );

        let mut resampled = Vec::<f32>::new();
        resampled.resize(to_len, 0.0);

        let upsample = self.from_rate < self.to_rate;
        if let Some(lpf) = &mut self.lpf {
            if upsample {
                for i in 0..to_len {
                    resampled[i] = lpf.filter(interpolator.interpolate(i, samples));
                }

                lpf.reset();

                for i in (0..to_len).rev() {
                    resampled[i] = lpf.filter(resampled[i]);
                }
            } else {
                let filtered = samples
                    .iter()
                    .copied()
                    .map(|sample| lpf.filter(sample))
                    .collect::<Vec<f32>>();

                lpf.reset();

                let filtered = filtered
                    .into_iter()
                    .rev()
                    .map(|sample| lpf.filter(sample))
                    .collect::<Vec<_>>();

                // let filtered = SampleList::new(&filtered, samples.bit_depth());

                resampled = (0..to_len)
                    .map(|index| interpolator.interpolate(index, &filtered))
                    .collect();
            }
        } else {
            resampled = (0..to_len)
                .map(|index| interpolator.interpolate(index, samples))
                .collect();
        }

        resampled
    }
}

pub struct ResamplerBuilder {
    from_rate: u32,
    to_rate: u32,
    mode: Option<InterpolationMode>,
    tension: Option<f32>,
    sinc_filter_size: Option<usize>,
    lpf: Option<FirLpf>,
    auto_lpf: bool,
}

impl ResamplerBuilder {
    pub fn new(from_rate: u32, to_rate: u32) -> Self {
        Self {
            from_rate,
            to_rate,
            mode: None,
            tension: None,
            sinc_filter_size: None,
            lpf: None,
            auto_lpf: true,
        }
    }

    pub fn interpolation_mode(mut self, mode: InterpolationMode) -> Self {
        self.mode = Some(mode);
        self
    }

    pub fn tension(mut self, tension: f32) -> Self {
        self.tension = Some(tension);
        self
    }

    pub fn sinc_filter_size(mut self, sinc_filter_size: usize) -> Self {
        self.sinc_filter_size = Some(sinc_filter_size);
        self
    }

    pub fn lpf(mut self, lpf: FirLpf) -> Self {
        self.lpf = Some(lpf);
        self
    }

    pub fn no_lpf(mut self) -> Self {
        self.auto_lpf = false;
        self
    }

    pub fn build(self) -> Resampler {
        let mode = self.mode.unwrap_or(DEFAULT_INTERPOLATION_MODE);
        let lpf = match self.lpf {
            Some(lpf) => Some(lpf),
            None if self.auto_lpf => match mode {
                InterpolationMode::Point | InterpolationMode::Linear => None,
                InterpolationMode::Cubic | InterpolationMode::Sinc => {
                    let upsample = self.from_rate < self.to_rate;
                    let (fir_sample_rate, fir_cutoff_freq) = if upsample {
                        (self.to_rate, self.from_rate as f32 / 2.0)
                    } else {
                        (self.from_rate, self.to_rate as f32 / 2.0)
                    };

                    Some(FirLpf::new(
                        DEFAULT_LPF_ORDER,
                        fir_sample_rate,
                        fir_cutoff_freq,
                    ))
                }
            },
            None => None,
        };

        let tension = self.tension.unwrap_or(DEFAULT_TENSION);
        let sinc_filter_size = self.sinc_filter_size.unwrap_or(DEFAULT_SINC_FILTER_SIZE);

        Resampler {
            from_rate: self.from_rate,
            to_rate: self.to_rate,
            mode,
            tension,
            sinc_filter_size,
            lpf,
        }
    }
}
