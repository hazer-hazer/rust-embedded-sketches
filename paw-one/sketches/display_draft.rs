#![no_std]
#![no_main]

extern crate paw_one;

use core::cell::RefCell;

use defmt::*;
use embassy_stm32::{dma::NoDma, peripherals, rcc::low_level::RccPeripheral, spi, time::Hertz};
use embedded_graphics::{
    geometry::Size,
    pixelcolor::{Rgb565, RgbColor},
};
use st7735_lcd::{Orientation, ST7735};
use {defmt, defmt_rtt as _, panic_probe as _};

use embassy_executor::Spawner;
use embedded_graphics::prelude::*;

const DISPLAY_SPI_FREQ: u32 = 64_000_000;

mod display;

const DISPLAY_BOX: Size = Size::new(160, 128);

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Program entered");

    let config = embassy_stm32::Config::default();

    info!("Configuring...");

    let p = embassy_stm32::init(config);

    let mut display_spi_cfg = spi::Config::default();
    display_spi_cfg.frequency = Hertz(DISPLAY_SPI_FREQ);
    display_spi_cfg.mode = spi::MODE_3;

    peripherals::SPI1::enable_and_reset();
    info!("SPI1 Freq: {}", peripherals::SPI1::frequency());

    let mosi = p.PA6;

    let cs = embassy_stm32::gpio::Output::new(
        p.PA6,
        embassy_stm32::gpio::Level::Low,
        embassy_stm32::gpio::Speed::Medium,
    );

    let dc = embassy_stm32::gpio::Output::new(
        p.PA9,
        embassy_stm32::gpio::Level::Low,
        embassy_stm32::gpio::Speed::Medium,
    );
    let rst = embassy_stm32::gpio::Output::new(
        p.PA10,
        embassy_stm32::gpio::Level::Low,
        embassy_stm32::gpio::Speed::Medium,
    );

    let spi = spi::Spi::new_txonly(p.SPI1, p.PA5, p.PA7, p.DMA1_CH1, NoDma, display_spi_cfg);

    let spi_bus = embassy_sync::blocking_mutex::Mutex::new(RefCell::new(spi));
    let display_spi = embassy_embedded_hal::shared_bus::blocking::spi::SpiDeviceWithConfig::new(
        spi_bus,
        cs,
        display_spi_cfg,
    );

    let mut display = ST7735::new(
        spi_device,
        dc,
        rst,
        true,
        false,
        DISPLAY_BOX.width,
        DISPLAY_BOX.height,
    );

    display.init(&mut embassy_time::Delay).unwrap();

    display.set_orientation(&Orientation::Landscape).unwrap();

    display.clear(Rgb565::BLACK).unwrap();
}
