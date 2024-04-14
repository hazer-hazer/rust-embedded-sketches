// /**
//  * Mostly taken from `dasp_sample` but much poorer
//  * https://github.com/RustAudio/dasp/blob/master/dasp_sample/src/lib.rs
//  */

// pub trait FromSample<S> {
//     fn from_sample(s: S) -> Self;
// }

// pub trait ToSample<S> {
//     fn to_sample(self) -> S;
// }

// impl<T, U> ToSample<U> for T
// where
//     U: FromSample<T>,
// {
//     fn to_sample(self) -> U {
//         U::from_sample(self)
//     }
// }

// macro_rules! impl_to_sample {
//     ($for: ty) => {
        
//     };
// }

// pub trait FromToSample<T>: ToSample<T> + FromSample<T> {}

// pub trait SignedSample:
//     Sample<Signed = Self>
//     + core::ops::Add<Output = Self>
//     + core::ops::Sub<Output = Self>
//     + core::ops::Neg<Output = Self>
// {
// }

// pub trait FloatSample:
//     Sample<Float = Self>
//     + SignedSample
//     + core::ops::Mul<Output = Self>
//     + core::ops::Div<Output = Self>
//     + FromToSample<f32>
// {
// }

// pub trait Sample: Copy + Clone + PartialOrd + PartialEq {
//     type Signed: SignedSample + FromToSample<Self>;
//     type Float: FloatSample + FromToSample<Self>;

//     // EQUILIBRIUM just not to mistake it with ZERO
//     const EQUILIBRIUM: Self;
// }

// macro_rules! impl_sample {
//     ($($for: ty, Signed: $signed: ty, Float: $float: ty, $equilibrium: expr),+) => {
//         $(
//             impl Sample for $for {
//                 type Signed = $signed;
//                 type Float = $float;

//                 const EQUILIBRIUM: Self = $equilibrium;
//             }
//         )+
//     };
// }

// impl_sample!(i8, Signed: i8, Float: f32, 0);
