use alloc::vec::Vec;
use micromath::F32Ext;

use crate::dsp::math::PI2;

// // Cannot use generics defaults in function signature thus create wrapper type
// pub struct WaveShapes<S: Sample = u16> {
//     phantom: PhantomData<S>,
// }

// TODO: Const generation when const generics math implemented
pub fn sine_wave(sample_rate: u32, pitch: f32, volume: f32) -> Vec<f32> {
    defmt::assert!(volume >= 0.0 && volume <= 1.0);
    defmt::assert!(pitch > 0.0);
    // Cannot go `for` in range of floats thus I allow only frequencies which are divisors of sample_rate
    let length = sample_rate as f32 / pitch;
    // assert!(length.fract() == 0.0);
    let length = length.floor();

    (0..length as usize).map(|i| (PI2 * i as f32 / length).sin() * volume)

    // num::range(T::zero(), T::from_f32(length).unwrap())
    //     .map(|i| {
    //         S::from_f32(
    //             (PI2 * i.to_f32().unwrap() / length).sin()
    //                 * (S::middle().to_f32().unwrap() - 1.0)
    //                 * volume
    //                 + S::middle().to_f32().unwrap(),
    //         )
    //         .unwrap()
    //     })
    //     .collect()
}
