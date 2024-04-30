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
    Each,
    Two,
    Four,
    Eight,
    Sixteen,
    ThirtyTwo,
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

    pub fn load<S: Sample + ToSample<f32>>(&mut self, buf: &[S]) {
        if self.full() {
            self.pointer = 0;
        }

        let chunk_size = match self.window {
            Window::Full => self.buf.len(),
            Window::Half => self.buf.len() / 2,
            Window::Quarter => self.buf.len() / 4,
            Window::Each => 1,
            Window::Two => 2,
            Window::Four => 4,
            Window::Eight => 8,
            Window::Sixteen => 16,
            Window::ThirtyTwo => 32,
            Window::Other(chunk_size) => chunk_size,
        };

        let avg_size = chunk_size as f32;
        let half_height = self.size.height as f32 / 2.0;

        for chunk in buf[self.channel_info.channel..]
            .chunks(chunk_size)
            .take(self.buf.len() - self.pointer)
        {
            let sum = chunk
                .iter()
                .step_by(self.channel_info.count)
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
        // TODO: Linear interpolation instead of point
        for x in 0..self.size.width as usize {
            let sample = self.buf[x * self.size.width as usize / self.buf.len()];
            let p = Point::new(x as i32, sample);
            Pixel(p + self.pos, self.color).draw(target)?;
        }

        Ok(())
    }
}
