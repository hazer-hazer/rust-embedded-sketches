use std::time::Duration;

use esp32_sandbox::audio::osc::simple_form::{SimpleFormSource, WaveForm};
use esp_idf_svc::{
    hal::{
        delay::TickType,
        gpio::AnyIOPin,
        i2s::{
            self,
            config::{StdClkConfig, StdConfig, StdGpioConfig, StdSlotConfig},
            I2sConfig, I2sDriver, I2sTxSupported,
        },
        peripherals::Peripherals,
    },
    log::set_target_level,
    sys::EspError,
};
use log::LevelFilter;

const SAMPLE_RATE: u32 = 16000;
const BIT_DEPTH: i2s::config::DataBitWidth = i2s::config::DataBitWidth::Bits32;

fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();
    set_target_level("*", LevelFilter::Trace).unwrap();

    std::env::set_var("RUST_BACKTRACE", "1");

    let pac = Peripherals::take().unwrap();

    // PCM5102A used, as I see by datasheet, it works with Philips I2S timing, i.e. one bit shift for WS
    let i2s_cfg = I2sConfig::default().dma_buffer_count(16);
    let clk_cfg = StdClkConfig::from_sample_rate_hz(SAMPLE_RATE);
    let slot_cfg = StdSlotConfig::philips_slot_default(BIT_DEPTH, i2s::config::SlotMode::Stereo);
    let gpio_cfg = StdGpioConfig::default();
    let std_config = StdConfig::new(i2s_cfg, clk_cfg, slot_cfg, gpio_cfg);

    let bclk = pac.pins.gpio4;
    let dout = pac.pins.gpio2;
    let ws = pac.pins.gpio15;
    let mut i2s =
        I2sDriver::new_std_tx(pac.i2s0, &std_config, bclk, dout, AnyIOPin::none(), ws).unwrap();

    i2s.tx_enable().unwrap();

    let mut wave =
        SimpleFormSource::infinite_mono(SAMPLE_RATE.into(), WaveForm::Sine, 240.0).into_iter();

    let wave_rendered = wave
        .map(|s| {
            let sample = ((s * 0.1 * i32::MAX as f32) as i32 as u32).to_be_bytes();
            [sample, sample]
        })
        .flatten()
        .flatten()
        .collect::<Vec<_>>();

    loop {
        i2s.write_all(&wave_rendered, TickType::new(100_000_000).into())
            .unwrap();
    }
}
