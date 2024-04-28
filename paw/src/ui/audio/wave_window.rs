use embedded_graphics::{
    geometry::{Point, Size},
    pixelcolor::PixelColor,
    Pixel,
};

use crate::{
    audio::source::Channel,
    dsp::sample::{Sample, ToSample},
};

// TODO: Window value range divisor

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
    channel_info: Channel,
    window: Window,
}

impl<const SIZE: usize, C: PixelColor> WaveWindow<SIZE, C> {
    pub fn new(pos: Point, size: Size, color: C, channel_info: Channel, window: Window) -> Self {
        Self {
            pos,
            size,
            buf: [0; SIZE],
            pointer: 0,
            color,
            channel_info,
            window,
        }
    }

    pub fn load<S: Sample + ToSample<i32>>(&mut self, buf: &[S]) {
        if self.full() {
            self.pointer = 0;
        }

        let chunk_size = match self.window {
            Window::Full => self.buf.len(),
            Window::Half => self.buf.len() / 2,
            Window::Quarter => self.buf.len() / 4,
            Window::Other(chunk_size) => chunk_size,
        };

        for chunk in buf.chunks(chunk_size).take(self.buf.len() - self.pointer) {
            let avg = chunk
                .iter()
                .skip(self.channel_info.channel)
                .step_by(self.channel_info.count)
                .copied()
                .fold(0, |sum, s| {
                    let sample = ToSample::<i32>::to_sample(s);
                    sum / 2
                        + (sample
                            + if sample < i32::EQUILIBRIUM {
                                i32::HIGH
                            } else {
                                i32::LOW
                            })
                            / 2
                });

            self.buf[self.pointer] = avg;
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

        // TODO: Linear interpolation instead of point
        for x in 0..self.size.width as usize {
            // let sample = self.buf[x * self.size.width as usize / self.buf.len()] as i64
            //     * self.size.height as i64
            //     / 2
            //     / i32::HIGH as i64;
            let sample = ToSample::<f32>::to_sample(self.buf[(x as f32 * width_factor) as usize])
                * self.size.height as f32
                / 2.;
            let p = Point::new(x as i32, sample as i32);
            Pixel(p + self.pos, self.color).draw(target)?;
        }

        Ok(())
    }
}
