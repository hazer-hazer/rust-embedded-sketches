// use core::{marker::PhantomData, time::Duration};

// use super::{AudioSource, AudioSourceProps, Sample};

// pub struct ZeroSource<S: Sample> {
//     props: AudioSourceProps,
//     marker: PhantomData<S>,
// }

// impl<S: Sample> ZeroSource<S> {
//     fn infinite_mono(sample_rate: u32) -> Self {
//         Self::new(1, sample_rate, None)
//     }

//     fn new(channels: u16, sample_rate: u32, duration: Option<Duration>) -> Self {
//         Self {
//             props: AudioSourceProps::new(channels, sample_rate, duration),
//             marker: PhantomData,
//         }
//     }
// }

// impl<S: Sample> AudioSource for ZeroSource<S> {
//     fn props(&self) -> super::AudioSourceProps {
//         self.props
//     }
// }

// impl<S: Sample> Iterator for ZeroSource<S> {
//     type Item = S;

//     fn next(&mut self) -> Option<Self::Item> {
//         if let Some(duration) = &mut self.props.duration {
//             if duration.tick() {
//                 Some(S::zero())
//             } else {
//                 None
//             }
//         } else {
//             Some(S::zero())
//         }
//     }
// }
