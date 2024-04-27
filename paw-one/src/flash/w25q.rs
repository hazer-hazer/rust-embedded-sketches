use core::{cell::Cell, marker::PhantomData};

use embedded_graphics::{
    draw_target::DrawTarget,
    geometry::{Dimensions, Point, Size},
    pixelcolor::{
        raw::{RawData, RawU16},
        PixelColor, Rgb565,
    },
    primitives::Rectangle,
    Drawable,
};
use embedded_graphics_framebuf::{backends::FrameBufferBackend, PixelIterator};
use embedded_hal::{blocking::spi::Transfer, digital::v2::OutputPin};

// TODO: Generic color

pub struct W25QStream<const BUFFER_SIZE: usize, SPI, CS: OutputPin> {
    w25q: w25q::series25::Flash<SPI, CS>,
    buf: [u8; BUFFER_SIZE],
    /// 24-bit address
    addr: u32,
    size: Size,
    pointer: usize,
}

impl<const BUFFER_SIZE: usize, SPI: Transfer<u8>, CS: OutputPin> W25QStream<BUFFER_SIZE, SPI, CS> {
    pub fn byte(&mut self) -> Result<u8, w25q::Error<SPI, CS>> {
        if self.pointer % BUFFER_SIZE == 0 {
            self.w25q
                .read(self.addr + self.pointer as u32, &mut self.buf)?;
        }
        let byte = self.buf[self.pointer % BUFFER_SIZE];
        self.pointer += 1;
        Ok(byte)
    }

    pub fn rgb565(&mut self) -> Result<Rgb565, w25q::Error<SPI, CS>> {
        Ok(RawU16::from_u32(u16::from_be_bytes([self.byte()?, self.byte()?]) as u32).into())
    }

    pub fn capacity(&self) -> usize {
        self.size.width as usize * self.size.height as usize
    }
}

impl<const BUFFER_SIZE: usize, SPI: Transfer<u8>, CS: OutputPin> Iterator
    for W25QStream<BUFFER_SIZE, SPI, CS>
{
    type Item = Rgb565;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pointer >= self.capacity() {
            None
        } else {
            self.rgb565().ok()
        }
    }
}

impl<const BUFFER_SIZE: usize, SPI, CS: OutputPin> Dimensions for W25QStream<BUFFER_SIZE, SPI, CS> {
    fn bounding_box(&self) -> Rectangle {
        Rectangle::new(Point::zero(), self.size)
    }
}

impl<const BUFFER_SIZE: usize, SPI: Transfer<u8>, CS: OutputPin> DrawTarget
    for W25QStream<BUFFER_SIZE, SPI, CS>
{
    type Color = Rgb565;

    type Error = ();

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = embedded_graphics::prelude::Pixel<Self::Color>>,
    {
        // self.w25q.write_bytes(self.addr, data)
        todo!()
    }
}

// impl<const BUFFER_SIZE: usize, SPI: Transfer<u8>, CS: OutputPin, C: PixelColor> FrameBufferBackend
//     for W25QStream<BUFFER_SIZE, SPI, CS, C>
// {
//     type Color = C;

//     fn set(&mut self, index: usize, color: Self::Color) {
//         todo!()
//     }

//     fn get(&self, index: usize) -> Self::Color {
//         // self.w25q.get_mut().read()
//         todo!()
//     }

//     fn nr_elements(&self) -> usize {
//         self.size
//     }
// }
