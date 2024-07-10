// use alloc::vec::Vec;

// use crate::dsp::sample::Sample;

// use super::AudioSource;

// pub struct Looped<S: AudioSource>
// where
//     S::Item: Sample,
// {
//     pub inner: S,
//     looping_index: Option<usize>,
//     buffer: Vec<S::Item>,
// }

// impl<S: AudioSource> Looped<S>
// where
//     S::Item: Sample,
// {
//     pub fn new(source: S) -> Self {
//         let buffer_size = source
//             .props()
//             .time
//             .total
//             .expect("Cannot create Buffered audio source from infinite audio source")
//             .samples;

//         Self {
//             inner: source,
//             looping_index: Some(0),
//             buffer: Vec::with_capacity(buffer_size as usize),
//         }
//     }

//     pub fn looping(&self) -> bool {
//         self.looping_index.is_some()
//     }

//     pub fn set_looping(&mut self, looping: bool) {
//         self.looping_index = looping.then(|| 0)
//     }
// }

// impl<S: AudioSource> AudioSource for Looped<S>
// where
//     S::Item: Sample,
// {
//     fn props(&self) -> super::AudioSourceProps {
//         self.inner.props()
//     }
// }

// impl<S: AudioSource> Iterator for Looped<S>
// where
//     S::Item: Sample,
// {
//     type Item = S::Item;

//     fn next(&mut self) -> Option<Self::Item> {
//         let sample = self.inner.next();
//         sample.or_else(|| self.looping_index.map(|index| self.buffer[index]))
//     }
// }
