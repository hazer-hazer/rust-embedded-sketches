use super::math::{F32Ext, SampleList};
use micromath::F32Ext as _;

#[derive(Clone, Copy)]
pub enum InterpolationMode {
    Point,
    Linear,
    Cubic,
    Sinc,
}

fn window(x: f32) -> f32 {
    (-x / 2.0 * x / 2.0).exp()
}

pub struct Interpolator {
    length: usize,
    scale_factor: f32,
    tangent_factor: f32,
    sinc_filter_size: usize,
    mode: InterpolationMode,
}

impl Interpolator {
    pub fn new(
        from_len: usize,
        to_len: usize,
        mode: InterpolationMode,
        tension: Option<f32>,
        sinc_filter_size: usize,
    ) -> Self {
        let length = from_len;
        let scale_factor: f32 = (from_len as f32 - 1.0) / to_len as f32;
        let tangent_factor = 1.0 - 0.0f32.max(1.0f32.min(tension.unwrap_or(0.0)));

        Self {
            length,
            scale_factor,
            tangent_factor,
            sinc_filter_size,
            mode,
        }
    }

    // TODO: Bit depth
    pub fn interpolate(&self, index: usize, samples: &[f32]) -> f32 {
        self._interpolate(index, samples)
    }

    fn _interpolate(&self, index: usize, samples: &[f32]) -> f32 {
        match self.mode {
            InterpolationMode::Point => self.point(index, samples),
            InterpolationMode::Linear => self.linear(index, samples),
            InterpolationMode::Cubic => self.cubic(index, samples),
            InterpolationMode::Sinc => self.sinc(index, samples),
        }
    }

    fn scale_index(&self, index: usize) -> (usize, f32, f32) {
        let scaled = self.scale_factor * index as f32;

        (scaled as usize, scaled - scaled.floor(), scaled)
    }

    fn get_tangent(&self, index: usize, samples: &[f32]) -> f32 {
        (samples.clipped_at(index + 1) - samples.prev_clipped_at(index)) / 2.0 * self.tangent_factor
    }

    // Modes //
    fn point(&self, index: usize, samples: &[f32]) -> f32 {
        samples.clipped_at(self.scale_index(index).0)
    }

    fn linear(&self, index: usize, samples: &[f32]) -> f32 {
        let (index, lerp_factor, _) = self.scale_index(index);
        samples
            .clipped_at(index)
            .lerp(samples.clipped_at(index + 1), lerp_factor)
    }

    fn cubic(&self, index: usize, samples: &[f32]) -> f32 {
        let (index, trans, _) = self.scale_index(index);

        let p = (samples.clipped_at(index), samples.clipped_at(index + 1));
        let m = (
            self.get_tangent(index, samples),
            self.get_tangent(index + 1, samples),
        );

        let trans2 = trans.powi(2);
        let trans3 = trans.powi(3);

        let result = p.0 * (2.0 * trans3 - 3.0 * trans2 + 1.0)
            + m.0 * (trans3 - 2.0 * trans2 + trans)
            + p.1 * (-2.0 * trans3 + 3.0 * trans2)
            + m.1 * (trans3 - trans2);

        result
    }

    fn sinc(&self, index: usize, samples: &[f32]) -> f32 {
        todo!()
        // let (index, _, indexF) = self.scale_index(index);

        // for i in index - self.sinc_filter_size + 1..index + self.sinc_filter_size + 1 {

        // }
    }
}
