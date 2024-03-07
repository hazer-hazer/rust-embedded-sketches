use core::mem::size_of;

use micromath::F32Ext as _;
use num::{traits::float::FloatCore, Bounded, FromPrimitive, Integer, ToPrimitive};
use num_traits::{Float, NumAssignOps};

pub const PI2: f32 = core::f32::consts::PI * 2.0;

pub trait IntExt: Bounded + FromPrimitive + Integer + ToPrimitive + Copy {
    // fn clamp(self, min: Self, max: Self) -> Self;
    fn lerp(self, to: Self, f: f32) -> f32;
    // TODO: Maybe add lerpi where factor is integer of some size, e.g. 0-1024 not 0.0-1.0
    // https://stackoverflow.com/questions/34098800/lerp-for-integers-or-in-fixed-point-math
    // fn lerpi(self, to: Self, f: Self) -> Self;
    fn middle() -> Self {
        // TODO: Optimize! remove freaking max_value and from_*
        Self::max_value() / Self::from_i8(2).unwrap()
    }

    fn remap<To: IntExt>(self) -> To {
        // To::min_value()
        //     + (self - Self::min_value()) * (To::max_value() - To::min_value())
        //         / (Self::max_value() - Self::min_value())

        // TODO: Check and Optimize.
        To::from_f64(
            self.to_f64().unwrap()
                * (To::max_value().to_f64().unwrap() / Self::max_value().to_f64().unwrap()),
        )
        .unwrap()
    }

    fn remap_to_range(self, to: (Self, Self)) -> Self {
        assert!(to.0 <= to.1);

        let value_range = (Self::max_value() - Self::min_value()).to_f64().unwrap();
        let to_range_len = (to.1 - to.0).to_f64().unwrap();
        Self::from_f64(
            self.to_f64().unwrap() * (to_range_len / value_range) + to.0.to_f64().unwrap(),
        )
        .unwrap()
    }
}

macro_rules! impl_int_ext {
    ($($tys: ty),+) => {
        $(
            impl IntExt for $tys {
                // fn clamp(self, min: Self, max: Self) -> Self {
                //     core::cmp::max(min, core::cmp::min(self, max))
                // }

                fn lerp(self, to: Self, f: f32) -> f32 {
                    self as f32 * (1.0 - f) + (to as f32 * f)
                }
            }
        )+
    };
}

impl_int_ext!(u8, u16, u32, u64, i8, i16, i32, i64, usize, isize);

pub trait F32Ext: Sized {
    fn lerp(self, to: Self, f: Self) -> Self;
}

impl F32Ext for f32 {
    fn lerp(self, to: Self, f: Self) -> Self {
        self * (1.0 - f) + (to * f)
    }
}

// pub trait F64Ext: Sized {
//     fn lerp(self, to: Self, f: Self) -> Self;
// }

// impl F64Ext for f64 {
//     fn lerp(self, to: Self, f: Self) -> Self {
//         self * (1.0 - f) + (to * f)
//     }
// }

// pub trait Sample:
//     Integer + Copy + ToPrimitive + IntExt + AsPrimitive<f32> + FromPrimitive + Bounded
// {
// }

// macro_rules! impl_sample_int {
//     ($($tys: ty),+) => {
//         $(
//             impl Sample for $tys {}
//         )+
//     };
// }

// impl_sample_int!(u8, u16, u32, u64, i8, i16, i32, i64, usize, isize);

// pub trait SampleList<S: Sample = u16>: core::ops::Index<usize, Output = S> {
//     fn len(&self) -> usize;

//     // Samples are clipped to zero
//     fn clipped_at(&self, index: usize) -> S {
//         if index < self.len() {
//             self[index]
//         } else {
//             S::zero()
//         }
//     }
// }

// impl<T: Sample> SampleList<T> for [T] {
//     fn len(&self) -> usize {
//         self.len()
//     }
// }

// impl<T: Sample, const S: usize> SampleList<T> for [T; S] {
//     fn len(&self) -> usize {
//         S
//     }
// }

// impl<T: Sample> SampleList<T> for Vec<T> {
//     fn len(&self) -> usize {
//         Vec::len(self)
//     }
// }

// pub type SampleList<'a, S: Sample> = &'a [S];

// TODO: Research usage of slice as it may be more convenient to use vec
// to allow in-place mutations without creation of new data
// #[derive(Clone, Copy)]
// pub struct SampleList<'a, T: SampleCore = f32> {
//     samples: &'a [Sample<T>],
//     // Bit depth is set as field but not as generic const
//     // to simplify dynamic creation of samples
//     bit_depth: u8,
// }

// impl<'a, T: SampleCore> SampleList<'a, T> {
//     pub fn new(samples: &'a [Sample<T>], bit_depth: u8) -> Self {
//         assert!(bit_depth % 8 == 0 && bit_depth as usize / 8 <= size_of::<Sample<T>>());

//         Self { samples, bit_depth }
//     }

//     pub fn bit_depth(&self) -> u8 {
//         self.bit_depth
//     }

//     pub fn len(&self) -> usize {
//         self.samples.len()
//     }

//     pub fn clipped_at(&self, index: usize) -> Sample<T> {
//         if index < self.len() {
//             self.samples[index]
//         } else {
//             Sample::zero()
//         }
//     }

//     pub fn prev_clipped_at(&self, index: usize) -> Sample<T> {
//         match index.checked_sub(1) {
//             Some(index) => self.clipped_at(index),
//             None => Sample::zero(),
//         }
//     }

//     pub fn iter(&self) -> impl Iterator<Item = &Sample<T>> {
//         self.samples.iter()
//     }
// }

// pub trait SampleCore: Float + NumAssignOps + F32Ext {
//     const PI: Self;
//     const PI2: Self;
// }

// impl SampleCore for f32 {
//     const PI: Self = f32::PI;
//     const PI2: Self = Self::PI * 2.0;
// }

// // impl SampleCore for f64 {
// //     const PI: Self = f64::PI;
// //     const PI2: Self = Self::PI * 2.0;
// // }

// // macro_rules! int_into_sample_core {
// //     ($($tys: ty),+) => {
// //         $(
// //             impl Into<SampleCore> for $tys {
// //                 fn into(self) -> SampleCore {
// //                     self as f32
// //                 }
// //             }
// //         )+
// //     };
// // }

// // int_into_sample_core!(u8, u16, u32, u64, i8, i16, i32, i64, usize, isize);

// #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
// pub struct Sample<T: SampleCore = f32>(T);

// macro_rules! impl_sample_ops {
//     ($($op: ty; $method: ident),+) => {
//         $(
//             impl<T: SampleCore> $op for Sample<T> {
//                 type Output = Self;

//                 fn $method(self, rhs: Self) -> Self::Output {
//                     Self::new(<$op>::$method(self.0, rhs.0))
//                 }
//             }

//             impl<T: SampleCore> $op<T> for Sample<T> {
//                 type Output = Self;

//                 fn $method(self, rhs: T) -> Self::Output {
//                     Self::new($op::$method(self.0, rhs))
//                 }
//             }
//         )+
//     };
// }

// impl_sample_ops!(core::ops::Add; add);

// // impl<T: SampleCore> core::ops::Add for Sample<T> {
// //     type Output = Self;

// //     fn add(self, rhs: Self) -> Self::Output {
// //         Self(self.0 + rhs.0)
// //     }
// // }

// // impl<T: SampleCore> core::ops::Add<T> for Sample<T> {
// //     type Output = Self;

// //     fn add(self, rhs: T) -> Self::Output {
// //         Self(self.0 + rhs)
// //     }
// // }

// // impl<T: SampleCore> core::ops::AddAssign<T> for Sample<T> {
// //     fn add_assign(&mut self, rhs: T) {
// //         self.0 += rhs;
// //     }
// // }

// impl<T: SampleCore> core::ops::Sub for Sample<T> {
//     type Output = Self;

//     fn sub(self, rhs: Self) -> Self::Output {
//         Self(self.0 - rhs.0)
//     }
// }

// impl<T: SampleCore> core::ops::Sub<T> for Sample<T> {
//     type Output = Self;

//     fn sub(self, rhs: T) -> Self::Output {
//         Self(self.0 - rhs)
//     }
// }

// impl<T: SampleCore> core::ops::Mul for Sample<T> {
//     type Output = Self;

//     fn mul(self, rhs: Self) -> Self::Output {
//         Self(self.0 * rhs.0)
//     }
// }

// impl<T: SampleCore> core::ops::Mul<T> for Sample<T> {
//     type Output = Self;

//     fn mul(self, rhs: T) -> Self::Output {
//         Self(self.0 * rhs)
//     }
// }

// impl<T: SampleCore> core::ops::Div for Sample<T> {
//     type Output = Self;

//     fn div(self, rhs: Self) -> Self::Output {
//         Self(self.0 / rhs.0)
//     }
// }

// impl<T: SampleCore> core::ops::Div<T> for Sample<T> {
//     type Output = Self;

//     fn div(self, rhs: T) -> Self::Output {
//         Self(self.0 / rhs)
//     }
// }

// impl<T: SampleCore> core::ops::Rem for Sample<T> {
//     type Output = Self;

//     fn rem(self, rhs: Self) -> Self::Output {
//         Self(self.0 % rhs.0)
//     }
// }

// impl<T: SampleCore> core::ops::Rem<T> for Sample<T> {
//     type Output = Self;

//     fn rem(self, rhs: T) -> Self::Output {
//         Self(self.0 % rhs)
//     }
// }

// impl<T: SampleCore> core::ops::Neg for Sample<T> {
//     type Output = Self;

//     fn neg(self) -> Self::Output {
//         Self(-self.0)
//     }
// }

// impl<T: SampleCore> From<T> for Sample<T> {
//     fn from(value: T) -> Self {
//         Self(value)
//     }
// }

// impl<T: SampleCore> Sample<T> {
//     pub fn try_new(value: T) -> Option<Self> {
//         if value >= -T::one() && value <= T::one() {
//             Some(Self(value))
//         } else {
//             None
//         }
//     }

//     pub fn new(value: T) -> Self {
//         assert!(value >= -T::one() && value <= T::one());
//         Self(value)
//     }

//     pub fn zero() -> Self {
//         Self::new(T::zero())
//     }

//     pub fn sin(self) -> Self {
//         Self::new(self.0.sin())
//     }

//     pub fn cos(self) -> Self {
//         Self::new(self.0.cos())
//     }

//     pub fn as_(self) -> T {
//         self.0
//     }

//     pub fn lerp(self, to: Self, f: T) -> Self {
//         Self::new(self.0.lerp(to.0, f))
//     }
// }

// // macro_rules! impl_inner {
// //     (impl $trait: ty => $for: ty { $($methods: ident),+ }) => {
// //         impl $trait for $for {
// //             $(fn $methods)
// //         }
// //     };
// // }
