// use alloc::vec::Vec;

// use crate::dsp::sample::Sample;

// use super::AudioSource;

// pub struct Buffered<S: AudioSource>
// where
//     S::Item: Sample,
// {
//     source: S,
//     buffer: Vec<S::Item>,
// }

// impl<S: AudioSource> Buffered<S>
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
//             source,
//             buffer: Vec::with_capacity(buffer_size as usize),
//         }
//     }
// }

// impl<S: AudioSource> AudioSource for Buffered<S>
// where
//     S::Item: Sample,
// {
//     fn props(&self) -> super::AudioSourceProps {
//         self.source.props()
//     }
// }

// impl<S: AudioSource> Iterator for Buffered<S>
// where
//     S::Item: Sample,
// {
//     type Item = S::Item;

//     fn next(&mut self) -> Option<Self::Item> {
//         if self.buffer.len()
//     }
// }
