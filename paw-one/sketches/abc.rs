#![no_std]
#![no_main]

extern crate paw_one;

use defmt::*;
use paw::{audio::osc::simple_form::SimpleFormSource, dsp::math::PI2};
use {defmt_rtt as _, panic_probe as _};

use embassy_executor::Spawner;
use embassy_stm32::{
    gpio::{self, low_level::Pin as _, AnyPin, Level, Output, Speed},
    rcc::RccPeripheral,
    sai,
    spi::{self, CkPin as _, MosiPin as _, Spi, Word, WsPin as _},
    time::Hertz,
};
use embassy_time::Timer;

const SAMPLE_RATE: u32 = 48_000;
const CHANNELS_COUNT: u32 = 2;
const BIT_DEPTH: u32 = 32;

const I2S_FREQ: u32 = SAMPLE_RATE * CHANNELS_COUNT * BIT_DEPTH;

/**
 * STM32G431CB I2S
 * PF0  - I2S2_WS
 * PF1  - I2S2_CK
 * PA4  - I2S3_WS
 * PB12 - I2S2_WS
 * PB13 - I2S2_CK
 * PB15 - I2S2_SD
 * PC6  - I2S2_MCK
 * PC7  - I2S3_MCK
 * PC9  - I2SCKIN
 * PA8  - I2S2_MCK
 * PA9  - I2S3_MCK
 * PA11 - I2S2_SD
 * PA12 - I2SCKIN
 * PA15 - I2S3_WS
 * PC10 - I2S3_CK
 * PC12 - I2S3_SD
 * PB3  - I2S3_CK
 * PB5  - I2S3_SD
 *
 * https://www.st.com/resource/en/datasheet/stm32g431cb.pdf
 *
 * I have 48-pin package, thus only A, B and C pin banks available.
 *
 * I2S2:
 * |  I2S2_MCK  |  I2S2_CK  |  I2S2_WS  |   I2S2_SD  |
 * | ---------- | --------- | --------- | ---------- |
 * |  PC6, PA8  | PF1, PB13 | PF0, PB12 | PB15, PA11 |
 *
 * I2S3:
 * |  I2S3_MCK  |  I2S3_CK  |  I2S3_WS  |   I2S3_SD  |
 * | ---------- | --------- | --------- | ---------- |
 * |  PC7, PA9  | PC10, PB3 | PA4, PA15 | PC12, PB5  |
 *
 *
 * SAI1 (single)
 * PE2  - SAI1_CK1, SAI1_MCLK_A
 * PE3  - SAI1_SD_B
 * PE4  - SAI1_D2, SAI1_FS_A
 * PE5  - SAI1_CK2, SAI1_SCK_A
 * PE6  - SAI1_D1, SAI1_SD_A
 * PF9  - SAI1_FS_B
 * PF10 - SAI1_D3
 * PC1  - SAI1_SD_A
 * PC3  - SAI1_D1, SAI1_SD_A
 * PA3  - SAI1_CK1, SAI1_MCLK_A
 * PA4  - SAI1_FS_B
 * PC5  - SAI1_D3
 * PE7  - SAI1_SD_B
 * PE8  - SAI1_SCK_B
 * PE9  - SAI1_FS_B
 * PE10 - SAI1_MCLK_B
 * PB10 - SAI1_SCK_A
 * PA8  - SAI1_CK2, SAI1_SCK_A
 * PA9  - SAI1_FS_A
 * PA10 - SAI1_D1, SAI1_SD_A
 * PA13 - SAI1_SD_B
 * PA14 - SAI1_FS_B
 * PD6  - SAI1_D1, SAI1_SD_A
 * PB3  - SAI1_SCK_B
 * PB4  - SAI1_MCLK_B
 * PB5  - SAI1_SD_B
 * PB6  - SAI1_FS_B
 * PB8  - SAI1_CK1, SAI1_MCLK_A
 * PB9  - SAI1_D2, SAI1_FS_A
 *
 * SAI1
 * SAI1_CK1: PE2, PA3, PB8
 * SAI1_CK2: PE5, PA8
 * |  Block \ PIN  |      MCLK     |       SCK      |           SD          |          FS          |
 * | ------------- | ------------- | -------------- | --------------------- | -------------------- |
 * |       A       |   PE2, PA3    | PE5, PB10, PA8 | PE6,PC1,PC3,PA10,PD6  |    PE4, PA9, PB9     |
 * |       B       |   PE10, PB4   | PE8, PB3       | PE3, PE7, PA13, PB5   | PF9,PA4,PE9,PA14,PB6 |
 */

/**
 * My SPI/I2S config
 * I2S  -> SPI pins
 * BCK  -> SCK  -> PB13
 * SD   -> MOSI -> PB15
 * WS   -> NSS (not present anywhere in embassy) -> PB12
 *
 * My SAI config
 * SCK - PA8
 * SD  - PA10
 * FS  - PA9
 */

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Program entered");

    let mut config = embassy_stm32::Config::default();

    {
        use embassy_stm32::rcc::*;

        // Set system clock to 76.8MHz, with HSI=16MHz set in PLL.
        // 76.8MHz is peeked cause it is divisible by 48KHz@2x32bit (3.072MHz) channels
        // 153.6MHz (two times larger) is also possible, but I'm only playing with STM32G431 with 170MHz much and maybe other board with lesser max clock frequency will be used, e.g. STM32G411 with 100MHz max.
        // HSI / 5 * 48 / 2 = 16MHz / 5 * 48 / 2
        // DIV2 is set for P, Q and R.
        // P is for ADC
        // Q is for USB, I2S23, SAI1, FDCAN, QSPI
        // R is for System Clock

        config.rcc.pll = Some(embassy_stm32::rcc::Pll {
            // 16MHz HSI
            source: PllSource::HSI,
            prediv_m: PllM::DIV5,
            mul_n: PllN::MUL48,
            div_p: Some(PllP::DIV2),
            div_q: Some(PllQ::DIV2),
            div_r: Some(PllR::DIV2),
        });

        // We need to set System Clock Mux to use our configured (above) Pll
        config.rcc.mux = ClockSrc::PLL;
    }

    info!("Configuring...");

    let p = embassy_stm32::init(config);

    info!("Hello World!");

    info!("KEK");

    // info!("Sys clock freq: {}", p.);

    // const BUFFER_SIZE: usize = 64;
    // let dma_buf = cortex_m::singleton!(: [u32; BUFFER_SIZE] = [0; BUFFER_SIZE]).unwrap();

    // let (sai_block_a, _) = sai::split_subblocks(p.SAI1);
    // let mut sai_cfg = sai::Config::default();
    // sai_cfg.mode = sai::Mode::Master;
    // sai_cfg.tx_rx = sai::TxRx::Transmitter;
    // sai_cfg.protocol = sai::Protocol::Free;
    // sai_cfg.slot_size = sai::SlotSize::DataSize;
    // sai_cfg.slot_count = sai::word::U4(2);
    // sai_cfg.first_bit_offset = sai::word::U5(0);
    // sai_cfg.slot_enable = 0b11;
    // sai_cfg.data_size = sai::DataSize::Data32;
    // sai_cfg.stereo_mono = sai::StereoMono::Stereo;
    // sai_cfg.bit_order = sai::BitOrder::MsbFirst;
    // sai_cfg.frame_sync_offset = sai::FrameSyncOffset::BeforeFirstBit;
    // sai_cfg.frame_sync_polarity = sai::FrameSyncPolarity::ActiveLow;
    // sai_cfg.frame_sync_active_level_length = sai::word::U7(32);
    // sai_cfg.frame_sync_definition = sai::FrameSyncDefinition::ChannelIdentification;
    // sai_cfg.frame_length = 64;
    // sai_cfg.fifo_threshold = sai::FifoThreshold::Quarter;
    // sai_cfg.clock_strobe = sai::ClockStrobe::Falling;

    // let mut sai = sai::Sai::new_asynchronous(
    //     sai_block_a,
    //     p.PA8,
    //     p.PA10,
    //     p.PA9,
    //     p.DMA1_CH1,
    //     dma_buf,
    //     sai_cfg,
    // );

    // sai.start();
    // info!("SAI Started");

    let mut spi_cfg = spi::Config::default();
    spi_cfg.frequency = Hertz::hz(I2S_FREQ);
    info!("Set I2S SPI frequency to {}", I2S_FREQ);

    let sd = p.PB15;
    let ws = p.PB12;
    let ck = p.PB13;

    sd.set_as_af(
        sd.af_num(),
        embassy_stm32::gpio::low_level::AFType::OutputPushPull,
    );
    sd.set_speed(gpio::Speed::VeryHigh);

    ws.set_as_af(ws.af_num(), gpio::low_level::AFType::OutputPushPull);
    ws.set_speed(crate::gpio::Speed::VeryHigh);

    ck.set_as_af(ck.af_num(), gpio::low_level::AFType::OutputPushPull);
    ck.set_speed(crate::gpio::Speed::VeryHigh);

    // I use txonly is only meaningful public constructor, actually, new_inner with all None's should be used
    let mut i2s_spi = Spi::new_txonly(p.SPI2, ck, sd, p.DMA1_CH1, p.DMA1_CH2, spi_cfg);

    // const I2S_SPI: embassy_stm32::pac::spi::Spi = embassy_stm32::pac::SPI2;

    let pclk =
        <embassy_stm32::peripherals::SPI2 as embassy_stm32::rcc::low_level::RccPeripheral>::frequency();

    embassy_stm32::pac::SPI2.i2spr().modify(|w| {
        w.set_i2sdiv(25);
        w.set_odd(embassy_stm32::pac::spi::vals::Odd::EVEN);

        // No master clock
        w.set_mckoe(false);
    });

    embassy_stm32::pac::SPI2.i2scfgr().modify(|w| {
        // Set clock inactive state to low level
        w.set_ckpol(embassy_stm32::pac::spi::vals::Ckpol::IDLELOW);

        // Set SPI to I2S mode
        w.set_i2smod(embassy_stm32::pac::spi::vals::Ismod::I2SMODE);

        // Set Philips standard I2S timings
        w.set_i2sstd(embassy_stm32::pac::spi::vals::Isstd::PHILIPS);

        // Set PCM Sync to short, only PCM standard with long sync needs LONG
        w.set_pcmsync(embassy_stm32::pac::spi::vals::Pcmsync::SHORT);

        // Set data len (bit depth) to 32
        w.set_datlen(embassy_stm32::pac::spi::vals::Datlen::THIRTYTWOBIT);

        // член? 🙀 Set channel length to 32
        w.set_chlen(embassy_stm32::pac::spi::vals::Chlen::THIRTYTWOBIT);

        // Set I2S to master transmitter
        w.set_i2scfg(embassy_stm32::pac::spi::vals::Iscfg::MASTERTX);

        // Enable I2s (why not :|)
        w.set_i2se(true);
    });

    info!("Starting main loop");

    // const BUFFER_SIZE: usize = 64;

    const SINE_FREQ: usize = 480;
    const PERIOD: usize = SAMPLE_RATE as usize / SINE_FREQ;
    let mut sine = [0; PERIOD * 2];
    let mut last_val = 0;
    for (i, s) in sine.iter_mut().enumerate() {
        use micromath::F32Ext;
        if i % 2 == 0 {
            *s = ((PI2 * i as f32 / PERIOD as f32).sin() * i32::MAX as f32) as i32 as u32;
            last_val = *s;
        } else {
            *s = last_val;
        }
    }

    // Period * 2Ch * Bytes
    let sine: [u8; PERIOD * 2 * 4] = unsafe { core::mem::transmute(sine) };

    info!("Send wave {:?}", sine);

    loop {
        i2s_spi.blocking_write(&sine).unwrap();
    }
}
