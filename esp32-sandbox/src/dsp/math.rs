use core::mem::size_of;

pub const PI2: f32 = core::f32::consts::PI * 2.0;

// pub trait IntExt: Bounded + FromPrimitive + Integer + ToPrimitive + Copy {
//     // fn clamp(self, min: Self, max: Self) -> Self;
//     fn lerp(self, to: Self, f: f32) -> f32;
//     // TODO: Maybe add lerpi where factor is integer of some size, e.g. 0-1024 not 0.0-1.0
//     // https://stackoverflow.com/questions/34098800/lerp-for-integers-or-in-fixed-point-math
//     // fn lerpi(self, to: Self, f: Self) -> Self;
//     fn middle() -> Self {
//         // TODO: Optimize! remove freaking max_value and from_*
//         Self::max_value() / Self::from_i8(2).unwrap()
//     }

//     fn remap<To: IntExt>(self) -> To {
//         // To::min_value()
//         //     + (self - Self::min_value()) * (To::max_value() - To::min_value())
//         //         / (Self::max_value() - Self::min_value())

//         // TODO: Check and Optimize.
//         To::from_f64(
//             self.to_f64().unwrap()
//                 * (To::max_value().to_f64().unwrap() / Self::max_value().to_f64().unwrap()),
//         )
//         .unwrap()
//     }

//     fn remap_to_range(self, to: (Self, Self)) -> Self {
//         assert!(to.0 <= to.1);

//         let value_range = (Self::max_value() - Self::min_value()).to_f64().unwrap();
//         let to_range_len = (to.1 - to.0).to_f64().unwrap();
//         Self::from_f64(
//             self.to_f64().unwrap() * (to_range_len / value_range) + to.0.to_f64().unwrap(),
//         )
//         .unwrap()
//     }
// }

// macro_rules! impl_int_ext {
//     ($($tys: ty),+) => {
//         $(
//             impl IntExt for $tys {
//                 // fn clamp(self, min: Self, max: Self) -> Self {
//                 //     core::cmp::max(min, core::cmp::min(self, max))
//                 // }

//                 fn lerp(self, to: Self, f: f32) -> f32 {
//                     self as f32 * (1.0 - f) + (to as f32 * f)
//                 }
//             }
//         )+
//     };
// }

// impl_int_ext!(u8, u16, u32, u64, i8, i16, i32, i64, usize, isize);

pub trait F32Ext: Sized {
    fn lerp(self, to: Self, f: Self) -> Self;
    fn remap_to_int_range(self, from: u32, to: u32) -> u32;
}

impl F32Ext for f32 {
    fn lerp(self, to: Self, f: Self) -> Self {
        self * (1.0 - f) + (to * f)
    }

    fn remap_to_int_range(self, from: u32, to: u32) -> u32 {
        ((self + 1.0) / 2.0 * (to - from) as f32) as u32 + from
    }
}

pub trait SampleList<S = f32>: core::ops::Index<usize> {
    fn clipped_at(&self, index: usize) -> S;
    fn prev_clipped_at(&self, index: usize) -> S;
}

impl SampleList for [f32] {
    fn clipped_at(&self, index: usize) -> f32 {
        if index < self.len() {
            self[index]
        } else {
            0.0
        }
    }

    fn prev_clipped_at(&self, index: usize) -> f32 {
        if index > 0 {
            self.clipped_at(index - 1)
        } else {
            0.0
        }
    }
}
