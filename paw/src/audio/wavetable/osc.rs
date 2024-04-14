// use crate::{
//     audio::source::{AudioSource, AudioSourceProps},
//     dsp::interpolator::InterpolationMode,
// };

// use super::wavetable::{self, WaveTable};

// pub struct WaveTableOsc {
//     // TODO: Maybe ref
//     wave_table: WaveTable,
//     props: AudioSourceProps,
//     index: f32,
//     step: f32,
//     interpolation_method: InterpolationMode,
// }

// impl WaveTableOsc {
//     pub fn infinite_mono_lerp(wave_table: WaveTable, sample_rate: u32) -> Self {
//         Self {
//             wave_table,
//             props: AudioSourceProps::infinite_mono(sample_rate),
//             index: 0.0,
//             step: 0.0,
//             interpolation_method: InterpolationMode::Linear,
//         }
//     }

//     pub fn set_freq(&mut self, freq: f32) -> &mut Self {
//         self.step = freq * self.wave_table.len() as f32 / self.sample_rate() as f32;
//         self
//     }

//     pub fn peek(&self) -> f32 {
//         match self.interpolation_method {
//             InterpolationMode::Point => todo!(),
//             InterpolationMode::Linear => {
//                 let trunc_index = self.index as usize;
//                 let next_index = (trunc_index + 1) % self.wave_table.len();
//                 let next_index_factor = self.index - trunc_index as f32;
//                 let index_factor = 1.0 - next_index_factor;

//                 index_factor * self.wave_table[trunc_index]
//                     + next_index_factor * self.wave_table[next_index]
//             }
//             InterpolationMode::Cubic => todo!(),
//             InterpolationMode::Sinc => todo!(),
//         }
//     }
// }

// impl Iterator for WaveTableOsc {
//     type Item = f32;

//     fn next(&mut self) -> Option<Self::Item> {
//         let sample = self.peek();

//         self.index = (self.index + self.step) % self.wave_table.len() as f32;

//         Some(sample)
//     }
// }

// impl AudioSource for WaveTableOsc {
//     fn props(&self) -> AudioSourceProps {
//         self.props
//     }
// }
