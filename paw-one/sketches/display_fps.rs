#![no_std]
#![no_main]

extern crate paw_one;

use crate::display::fps::FPS;
use defmt::*;
use embassy_time::{Duration, Instant};
use embedded_text::{
    alignment::HorizontalAlignment,
    style::{HeightMode, TextBoxStyleBuilder},
    TextBox,
};

use {defmt_rtt as _, panic_probe as _};

use core::fmt::Write;
use embassy_executor::Spawner;
use embassy_stm32::{dma::NoDma, gpio::Output, time::Hertz};
use embedded_graphics::{
    mock_display::ColorMapping,
    mono_font::{ascii::FONT_6X10, MonoTextStyle, MonoTextStyleBuilder},
    pixelcolor::{BinaryColor, Rgb565},
    prelude::*,
    primitives::Rectangle,
    text::Text,
};

mod display;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Program entered");

    let mut config = embassy_stm32::Config::default();

    {
        use embassy_stm32::rcc::*;
        config.rcc.pll = Some(Pll {
            source: PllSource::HSI,
            prediv_m: PllM::DIV4,
            mul_n: PllN::MUL85,
            div_p: None,
            div_q: None,
            // Main system clock at 170 MHz
            div_r: Some(PllR::DIV2),
        });
        config.rcc.mux = ClockSrc::PLL;
    }

    info!("Configuring...");

    let p = embassy_stm32::init(config);

    let sck = p.PA5;
    let mosi = p.PA7;
    let cs = Output::new(
        p.PA6,
        embassy_stm32::gpio::Level::High,
        embassy_stm32::gpio::Speed::High,
    );
    let rst = Output::new(
        p.PA10,
        embassy_stm32::gpio::Level::High,
        embassy_stm32::gpio::Speed::High,
    );
    let dc = Output::new(
        p.PA4,
        embassy_stm32::gpio::Level::High,
        embassy_stm32::gpio::Speed::High,
    );

    let _led = Output::new(
        p.PA12,
        embassy_stm32::gpio::Level::High,
        embassy_stm32::gpio::Speed::Low,
    );

    let mut display_spi_cfg = embassy_stm32::spi::Config::default();
    display_spi_cfg.frequency = Hertz(64_000_000);
    let spi =
        embassy_stm32::spi::Spi::new_txonly(p.SPI1, sck, mosi, p.DMA1_CH1, NoDma, display_spi_cfg);

    let spi_device = embedded_hal_bus::spi::ExclusiveDevice::new_no_delay(spi, cs);

    let mut display = st7735_lcd::ST7735::new(spi_device, dc, rst, true, false, 128, 160);

    display.init(&mut embassy_time::Delay).unwrap();
    display
        .set_orientation(&st7735_lcd::Orientation::Landscape)
        .unwrap();

    display.clear(Rgb565::BLACK).unwrap();
    display.clear(Rgb565::RED).unwrap();

    let character_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(Rgb565::GREEN)
        .background_color(Rgb565::WHITE)
        .build();
    let textbox_style = TextBoxStyleBuilder::new()
        .height_mode(HeightMode::FitToText)
        .alignment(HorizontalAlignment::Left)
        .paragraph_spacing(0)
        .trailing_spaces(true)
        .build();

    // Specify the bounding box. Note the 0px height. The `FitToText` height mode will
    // measure and adjust the height of the text box in `into_styled()`.
    let bounds = Rectangle::new(Point::new(0, 0), Size::new(50, 10));

    // let fps_text_style = ;
    let mut fps = FPS::new();
    loop {
        let mut fps_text = heapless::String::<100>::new();
        core::write!(fps_text, "{}fps", fps.value()).unwrap();

        TextBox::with_textbox_style(&fps_text, bounds, character_style, textbox_style)
            .draw(&mut display)
            .unwrap();
    }
}
