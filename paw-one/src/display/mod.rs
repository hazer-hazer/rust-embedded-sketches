use display_interface::{DataFormat, DisplayError, WriteOnlyDataCommand};
use embedded_hal_1::{
    digital::{self, OutputPin},
    spi::{self, SpiDevice},
};

pub mod fps;
// pub mod st7735;

// pub struct SPIDeviceInterface<SPI, DC> {
//     spi: SPI,
//     dc: DC,
// }

// impl<SPI, DC> SPIDeviceInterface<SPI, DC>
// where
//     SPI: spi::SpiDevice,
//     DC: digital::OutputPin,
// {
//     pub fn new(spi: SPI, dc: DC) -> Self {
//         Self { spi, dc }
//     }
// }

// impl<SPI, DC> WriteOnlyDataCommand for SPIDeviceInterface<SPI, DC>
// where
//     SPI: SpiDevice,
//     DC: OutputPin,
// {
//     fn send_commands(
//         &mut self,
//         cmd: display_interface::DataFormat<'_>,
//     ) -> Result<(), display_interface::prelude::_display_interface_DisplayError> {
//         self.dc.set_low().map_err(|_| DisplayError::DCError);
//         send_u8(self, cmd).map_err(|_| DisplayError::BusWriteError)
//     }

//     fn send_data(
//         &mut self,
//         buf: display_interface::DataFormat<'_>,
//     ) -> Result<(), display_interface::prelude::_display_interface_DisplayError> {
//         self.dc.set_high().map_err(|_| DisplayError::DCError);
//         send_u8(self, buf).map_err(|_| DisplayError::BusWriteError)
//     }
// }

// fn send_u8<T: SpiDevice>(spi: &mut T, words: DataFormat<'_>) -> Result<(), T::Error> {
//     use byte_slice_cast::*;
//     match words {
//         DataFormat::U8(slice) => spi.write(slice),
//         DataFormat::U16(slice) => spi.write(slice),
//         DataFormat::U16LE(slice) => {
//             for v in slice.as_mut() {
//                 *v = v.to_le();
//             }
//             spi.write(slice.as_byte_slice())
//         }
//         DataFormat::U16BE(slice) => {
//             for v in slice.as_mut() {
//                 *v = v.to_be();
//             }
//             spi.write(slice.as_byte_slice())
//         }
//         DataFormat::U8Iter(iter) => {
//             let mut buf = [0; 32];
//             let mut i = 0;

//             for v in iter.into_iter() {
//                 buf[i] = v;
//                 i += 1;

//                 if i == buf.len() {
//                     spi.write(&buf)?;
//                     i = 0;
//                 }
//             }

//             if i > 0 {
//                 spi.write(&buf[..i])?;
//             }

//             Ok(())
//         }
//         DataFormat::U16LEIter(iter) => {
//             let mut buf = [0; 32];
//             let mut i = 0;

//             for v in iter.map(u16::to_le) {
//                 buf[i] = v;
//                 i += 1;

//                 if i == buf.len() {
//                     spi.write(&buf.as_byte_slice())?;
//                     i = 0;
//                 }
//             }

//             if i > 0 {
//                 spi.write(&buf[..i].as_byte_slice())?;
//             }

//             Ok(())
//         }
//         DataFormat::U16BEIter(iter) => {
//             let mut buf = [0; 64];
//             let mut i = 0;
//             let len = buf.len();

//             for v in iter.map(u16::to_be) {
//                 buf[i] = v;
//                 i += 1;

//                 if i == len {
//                     spi.write(&buf.as_byte_slice())?;
//                     i = 0;
//                 }
//             }

//             if i > 0 {
//                 spi.write(&buf[..i].as_byte_slice())?;
//             }

//             Ok(())
//         }
//         _ => unimplemented!(),
//     }
// }
