use num::Bounded;
use num::Saturating;
use num_traits::SaturatingSub;

// /**
//  * Mostly taken from `dasp_sample` but much poorer
//  * https://github.com/RustAudio/dasp/blob/master/dasp_sample/src/lib.rs
//  */
pub trait FromSample<S> {
    fn from_sample(s: S) -> Self;
}

pub trait ToSample<S> {
    fn to_sample(self) -> S;
}

impl<T, U> ToSample<U> for T
where
    U: FromSample<T>,
{
    #[inline(always)]
    fn to_sample(self) -> U {
        U::from_sample(self)
    }
}

impl<T> FromSample<T> for T {
    #[inline(always)]
    fn from_sample(s: T) -> Self {
        s
    }
}

// pub trait ToBoundedSample: Bounded + Integer {
//     fn to_signed_sample<U: Bounded + Integer>(self) -> U {
//         let range = Self::max_value() - Self::min_value();
//         let to_range = U::max_value() - U::min_value();
//         (self - Self::min_value()) * to_range / range + U::min_value()
//     }
// }

pub trait FromToSample<T>: ToSample<T> + FromSample<T> {}

impl<T: FromSample<U>, U> FromToSample<U> for T where U: FromSample<T> {}

pub trait SignedSample: Sample<Signed = Self> + core::ops::Neg<Output = Self> {}

macro_rules! impl_signed_sample {
    ($($ty: ty),+) => {
        $(impl SignedSample for $ty {})+
    };
}

impl_signed_sample!(i8, i16, i32, i64, f32);

pub trait FloatSample:
    Sample<Float = Self>
    + SignedSample
    + core::ops::Mul<Output = Self>
    + core::ops::Div<Output = Self>
    + FromToSample<f32>
{
}

impl FloatSample for f32 {}

pub trait Sample:
    Copy
    + Clone
    + PartialOrd
    + PartialEq
    + core::ops::Add<Output = Self>
    + core::ops::Sub<Output = Self>
    + Bounded
{
    type Signed: SignedSample + FromToSample<Self>;
    type Float: FloatSample + FromToSample<Self>;

    const LOW: Self;

    // EQUILIBRIUM just not to mistake it with ZERO
    const EQUILIBRIUM: Self;

    const HIGH: Self;

    fn to_signed(self) -> Self::Signed {
        self.to_sample()
    }

    fn to_float(self) -> Self::Float {
        self.to_sample()
    }

    // fn invert(self) -> Self::Signed {
    //     Self::Signed::EQUILIBRIUM.saturating_sub(self.to_signed())
    // }

    fn points() -> usize;
}

macro_rules! impl_sample {
    ($(
        $for: ty, Signed: $signed: ty, Float: $float: ty, $low: expr, $eq: expr, $high: expr, from {
            $($sample_ident: ident: $from: ty => $body: expr),+ $(,)?
        }
    ),+ $(,)?) => {
        $(
            impl Sample for $for {
                type Signed = $signed;
                type Float = $float;

                const LOW: Self = $low;
                const EQUILIBRIUM: Self = $eq;
                const HIGH: Self = $high;

                fn points() -> usize {
                    (Self::max_value() - Self::min_value()) as usize
                }
            }

            $(
                impl FromSample<$from> for $for {
                    #[inline(always)]
                    fn from_sample($sample_ident: $from) -> Self {
                        $body
                    }
                }
            )+
        )+
    };
}

impl_sample! {
    i8, Signed: i8, Float: f32, -128, 0, 127, from {
        s: i16 => (s >> 8) as i8,
        s: i32 => (s >> 24) as i8,
        s: i64 => (s >> 56) as i8,
        s: u8 => if s < 128 {
            s as i8 - 127 - 1
        } else {
            (s - 128) as i8
        },
        s: u16 => ((s >> 8) as u8).to_sample(),
        s: u32 => ((s >> 24) as u8).to_sample(),
        s: u64 => ((s >> 56) as u8).to_sample(),
        s: f32 => (s * 128.0) as i8,
    },
    i16, Signed: i16, Float: f32, -32_768, 0, 32_767, from {
        s: i8 => (s as i16) << 8,
        s: i32 => (s >> 16) as i16,
        s: i64 => (s >> 48) as i16,
        s: u8 => (s as i16 - 128) << 8,
        s: u16 => if s < 32_768 {
            s as i16 - 32_767 - 1
        } else {
            (s - 32_768) as i16
        },
        s: u32 => ((s >> 16) as u16).to_sample(),
        s: u64 => ((s >> 48) as u16).to_sample(),
        s: f32 => (s * 32_768.0) as i16,
    },
    i32, Signed: i32, Float: f32, -2_147_483_648, 0, 2_147_483_647, from {
        s: i8 => (s as i32) << 24,
        s: i16 => (s as i32) << 16,
        s: i64 => (s >> 32) as i32,
        s: u8 => (s as i32 - 128) << 24,
        s: u16 => (s as i32 - 32_768) << 16,
        s: u32 => if s < 2_147_483_648 {
            s as i32 - 2_147_483_647 - 1
        } else {
            (s - 2_147_483_648) as i32
        },
        s: u64 => ((s >> 32) as u32).to_sample(),
        s: f32 => (s * 2_147_483_648.0) as i32,
    },
    i64, Signed: i64, Float: f32, -9_223_372_036_854_775_808, 0, 9_223_372_036_854_775_807, from {
        s: i8 => (s as i64) << 56,
        s: i16 => (s as i64) << 48,
        s: i32 => (s as i64) << 32,
        s: u8 => (s as i64 - 128) << 56,
        s: u16 => (s as i64 - 32_768) << 48,
        s: u32 => (s as i64 - 2_147_483_648) << 32,
        s: u64 => if s < 9_223_372_036_854_775_808 {
            s as i64 - 9_223_372_036_854_775_807 - 1
        } else {
            (s - 9_223_372_036_854_775_808) as i64
        },
        s: f32 => (s * 9_223_372_036_854_775_808.0) as i64,
    },
    u8, Signed: i8, Float: f32, 0, 128, 255, from {
        s: i8 => if s < 0 {
            (s + 127 + 1) as u8
        } else {
            (s as u8) + 128
        },
        s: i16 => ((s >> 8) as i8).to_sample(),
        s: i32 => ((s >> 24) as i8).to_sample(),
        s: i64 => ((s >> 56) as i8).to_sample(),
        s: u16 => (s >> 8) as u8,
        s: u32 => (s >> 24) as u8,
        s: u64 => (s >> 56) as u8,
        s: f32 => ToSample::<i8>::to_sample(s).to_sample(),
    },
    u16, Signed: i16, Float: f32, 0, 32_768, 65_535, from {
        s: i8 => if s < 0 {
            ((s + 127 + 1) as u16) << 8
        } else {
            (s as u16 + 128) << 8
        },
        s: i16 => if s < 0 {
            (s + 32_767 + 1) as u16
        } else {
            s as u16 + 32_768
        },
        s: i32 => ((s >> 16) as i16).to_sample(),
        s: i64 => ((s >> 48) as i16).to_sample(),
        s: u8 => (s as u16) << 8,
        s: u32 => (s >> 16) as u16,
        s: u64 => (s >> 48) as u16,
        s: f32 => ToSample::<i16>::to_sample(s).to_sample(),
    },
    u32, Signed: i32, Float: f32, 0, 2_147_483_648, 4_294_967_295, from {
        s: i8 => if s < 0 {
            ((s + 127 + 1) as u32) << 24
        } else {
            (s as u32 + 128) << 24
        },
        s: i16 => if s < 0 {
            ((s + 32_767 + 1) as u32) << 16
        } else {
            ((s as u32) + 32_768) << 16
        },
        s: i32 => if s < 0 {
            (s + 2_147_483_647 + 1) as u32
        } else {
            s as u32 + 2_147_483_648
        },
        s: i64 => ((s >> 32) as i32).to_sample(),
        s: u8 => (s as u32) << 24,
        s: u16 => (s as u32) << 16,
        s: u64 => (s >> 32) as u32,
        s: f32 => ToSample::<i32>::to_sample(s).to_sample(),
    },
    u64, Signed: i64, Float: f32, 0, 9_223_372_036_854_775_808, 18_446_744_073_709_551_615, from {
        s: i8 => if s < 0 {
            ((s + 127 + 1) as u64) << 56
        } else {
            (s as u64 + 128) << 56
        },
        s: i16 => if s < 0 {
            ((s + 32_767 + 1) as u64) << 48
        } else {
            ((s as u64) + 32_768) << 48
        },
        s: i32 => if s < 0 {
            ((s + 2_147_483_647 + 1) as u64) << 32
        } else {
            (s as u64) + 2_147_483_648 << 32
        },
        s: i64 => if s < 0 {
            (s + 9_223_372_036_854_775_807 + 1) as u64
        } else {
            s as u64 + 9_223_372_036_854_775_808
        },
        s: u8 => (s as u64) << 56,
        s: u16 => (s as u64) << 48,
        s: u32 => (s as u64) << 32,
        s: f32 => ToSample::<i64>::to_sample(s).to_sample(),
    },
    f32, Signed: f32, Float: f32, -1.0, 0.0, 1.0, from {
        s: i8 => s as f32 / 128.0,
        s: i16 => s as f32 / 32_768.0,
        s: i32 => s as f32 / 2_147_483_648.0,
        s: i64 => s as f32 / 9_223_372_036_854_775_808.0,
        s: u8 => ToSample::<i8>::to_sample(s).to_sample(),
        s: u16 => ToSample::<i16>::to_sample(s).to_sample(),
        s: u32 => ToSample::<i32>::to_sample(s).to_sample(),
        s: u64 => ToSample::<i64>::to_sample(s).to_sample(),
    },
}

// TODO: Bounded sample type with Min, Max. Convenient for displaying for example

// #[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
// pub struct BoundedSample<S: Sample> {
//     bounds: (S, S),
// }

// impl<S: SignedSample + Zero + core::ops::Neg> SignedSample for BoundedSample<S> {}

// impl<S> Sample for BoundedSample<S>
// where
//     S: Zero + Sample,
// {
//     type Signed = BoundedSample<S::Signed>;
//     type Float = BoundedSample<S::Float>;

//     const EQUILIBRIUM: Self = S::EQUILIBRIUM;

//     fn points(self) -> <Self as core::ops::Sub<Self>>::Output
//     where
//         Self: Bounded + core::ops::Sub,
//     {
//         self.bounds.1 - self.bounds.0
//     }
// }

// impl<S: Sample> BoundedSample<S> {
//     pub fn new(from: S, to: S) -> Self {
//         assert!(from < to);
//         Self { bounds: (from, to) }
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;

    // Error threshold for int to float conversion (inclusive)
    const F32_CONV_ERROR_THRESHOLD: f32 = 0.01;

    // macro_rules! ints {
    //     (signed) => {
    //         i8, i16, i32, i64
    //     };

    //     (unsigned) => {
    //         u8, u16, u32, u64
    //     };

    //     (all) => {
    //         ints!(signed), ints!(unsigned)
    //     };
    // }

    macro_rules! identity_conv {
        ($($ty: ty),+) => {
            $({
                let val: $ty = 2 as $ty;
                let val_: $ty = val.to_sample();
                assert_eq!(val, val_);
            })*
        };
    }

    #[test]
    fn identity() {
        identity_conv!(i8, i16, i32, i64, u8, u16, u32, u64, f32);
    }

    macro_rules! test_int_to_f32 {
        ($name: ident: $int: ty, $int_val: expr => $float_val: expr) => {
            let $name: $int = $int_val;
            let to_f32: f32 = $name.to_sample();
            let error = ($float_val - to_f32).abs();
            assert!(error <= F32_CONV_ERROR_THRESHOLD, "check {} {}: {} conversion failed: {} !=> {}f32. Result = {}. Error = {}", stringify!($name), stringify!($int_val), stringify!($int), $int_val, $float_val, to_f32, error);
        };

        ($($int: ty),+) => {
            $({
                test_int_to_f32!(low: $int, <$int>::LOW => -1.0);
                test_int_to_f32!(equilibrium: $int, <$int>::EQUILIBRIUM => f32::EQUILIBRIUM);
                test_int_to_f32!(high: $int, <$int>::HIGH => 1.0);
            })+
        };
    }

    #[test]
    fn int_to_f32() {
        test_int_to_f32!(i8, i16, i32, i64, u8, u16, u32, u64);
    }

    macro_rules! test_bounds {
        ($($ty: ty: ($low: expr, $eq: expr, $high: expr)),+) => {
            $({
                assert_eq!(<$ty>::LOW, $low);
                assert_eq!(<$ty>::EQUILIBRIUM, $eq);
                assert_eq!(<$ty>::HIGH, $high);
            })+
        };
    }

    #[test]
    fn bounds() {
        test_bounds! {
            i8: (-128, 0, 127),
            i16: (-32_768, 0, 32_767),
            i32: (-2_147_483_648, 0, 2_147_483_647),
            i64: (-9_223_372_036_854_775_808, 0, 9_223_372_036_854_775_807),
            u8: (0, 128, 255),
            u16: (0, 32_768, 65_535),
            u32: (0, 2_147_483_648, 4_294_967_295),
            u64: (0, 9_223_372_036_854_775_808, 18_446_744_073_709_551_615),
            f32: (-1.0, 0.0, 1.0)
        };
    }

    // macro_rules! f32_equilibrium_to_int {
    //     ($($int: ty),+) => {
    //         $({
    //             let to_int_eq: $int = f32::EQUILIBRIUM.to_sample();
    //             assert_eq!(to_int_eq, <$int>::EQUILIBRIUM, "check valid equilibrium conversion f32 => {}. f32::EQUILIBRIUM ({} => {}) !=> {}::EQUILIBRIUM ({})", stringify!($int), f32::EQUILIBRIUM, to_int_eq, stringify!($int), <$int>::EQUILIBRIUM);
    //         })+
    //     };
    // }

    // #[test]
    // fn f32_equilibrium_to_int() {
    //     f32_equilibrium_to_int!(i8, i16, i32, i64, u8, u16, u32, u64);
    // }

    macro_rules! test_f32_to_int {
        ($name: ident: $int: ty, $float_val: expr => $int_val: expr) => {
            let $name: $int = $float_val.to_sample();
            assert_eq!($name, $int_val, "conversion {}: f32 failed: {}: f32 !=> {}: {}", stringify!($name), $float_val, stringify!($int), $int_val);
        };

        ($($int: ty),+) => {
            $({
                test_f32_to_int!(low: $int, f32::LOW => <$int>::LOW);
                test_f32_to_int!(equilibrium: $int, f32::EQUILIBRIUM => <$int>::EQUILIBRIUM);
                test_f32_to_int!(high: $int, f32::HIGH => <$int>::HIGH);
            })+
        };
    }

    #[test]
    fn f32_to_int() {
        test_f32_to_int!(i8, i16, i32, i64, u8, u16, u32, u64);
    }

    // macro_rules! test_int_uint_bounds {
    //     ($(int: ty, uint: ty),+) => {
    //         $({
    //             let
    //             assert_eq!()
    //         })+
    //     };
    // }

    // #[test]
    // fn int_uint_bounds() {}
}
