use defmt::info;
use embedded_graphics::{
    geometry::{Point, Size},
    pixelcolor::{PixelColor, Rgb565, RgbColor},
    primitives::{Line, Polyline, Primitive, PrimitiveStyle, StyledDrawable},
    transform::Transform,
    Pixel,
};
use micromath::F32Ext;
use num::Saturating;
use paw::dsp::sample::ToSample;
use paw::{audio::source::Channel, dsp::sample::Sample};

#[derive(Clone, Copy)]
pub enum Window {
    Full,
    Half,
    Quarter,
    Other(usize),
}

pub struct WaveWindow<const SIZE: usize, C: PixelColor> {
    pos: Point,
    size: Size,
    buf: [i32; SIZE],
    pointer: usize,
    color: C,
}

impl<const SIZE: usize, C: PixelColor> WaveWindow<SIZE, C> {
    pub fn new(pos: Point, size: Size, color: C) -> Self {
        Self {
            pos,
            size,
            buf: [0; SIZE],
            pointer: 0,
            color,
        }
    }

    pub fn load<S: Sample + ToSample<f32>>(&mut self, buf: &[S], channel: Channel, window: Window) {
        if self.full() {
            self.pointer = 0;
        }

        let chunk_size = match window {
            Window::Full => self.buf.len(),
            Window::Half => self.buf.len() / 2,
            Window::Quarter => self.buf.len() / 4,
            Window::Other(chunk_size) => chunk_size,
        };

        let avg_size = chunk_size as f32;
        let half_height = self.size.height as f32 / 2.0;

        for chunk in buf.chunks(chunk_size).take(self.buf.len() - self.pointer) {
            let sum = chunk
                .iter()
                .skip(channel.channel)
                .step_by(channel.count)
                .copied()
                .fold(0.0, |sum, s| {
                    let sample = ToSample::<f32>::to_sample(s);
                    sum + if sample < 0.0 {
                        sample + 1.0
                    } else {
                        sample - 1.0
                    }
                });

            self.buf[self.pointer] = (sum / avg_size * half_height + half_height) as i32;
            self.pointer += 1;
        }
    }

    pub fn full(&self) -> bool {
        self.pointer >= self.buf.len()
    }
}

impl<const SIZE: usize, C: PixelColor> embedded_graphics::Drawable for WaveWindow<SIZE, C> {
    type Color = C;
    type Output = ();

    fn draw<D>(&self, target: &mut D) -> Result<Self::Output, D::Error>
    where
        D: embedded_graphics::prelude::DrawTarget<Color = Self::Color>,
    {
        let width_factor = self.buf.len() as f32 / self.size.width as f32;
        let half_height = self.size.height as f32 / 2.0;
        // let vertices = self
        //     .buf
        //     .iter()
        //     .enumerate()
        //     .map(|(i, &s)| {
        //         Point::new(
        //             (i as f32 * width_factor) as i32,
        //             ((1.0 - ToSample::<f32>::to_sample(s)) * height_factor) as i32,
        //         )
        //     })
        //     .collect::<heapless::Vec<_, 512>>();

        // for (i, &s) in self.buf.iter().enumerate() {
        //     let p = Point::new(
        //         (i as f32 * width_factor) as i32,
        //         ((ToSample::<f32>::to_sample(s) + 1.0) * height_factor) as i32,
        //     );
        //     Pixel(p, Rgb565::WHITE).draw(target)?;
        // }

        // Ok(())

        // Polyline::new(&vertices)
        //     .into_styled(self.style)
        //     .translate(self.pos)
        //     .draw(target)

        // for x in 0..self.size.width as usize {
        //     let sample = self.buf[(x as f32 * width_factor) as usize + self.channel];
        //     let p = Point::new(
        //         x as i32,
        //         0 - (ToSample::<f32>::to_sample(sample) * half_height) as i32
        //             + self.size.height as i32 / 2,
        //     );
        //     Pixel(p + self.pos, self.color).draw(target)?;
        // }

        // TODO: Linear interpolation instead of point
        for x in 0..self.size.width as usize {
            let sample = self.buf[(x as f32 * width_factor) as usize];
            let p = Point::new(x as i32, sample);
            Pixel(p + self.pos, self.color).draw(target)?;
        }

        Ok(())
    }
}
