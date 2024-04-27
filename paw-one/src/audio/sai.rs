use defmt::info;
use embassy_stm32::{peripherals::SAI1, sai};

use super::AudioOutput;

const BUFFER_SIZE: usize = 512;

// Just trying to avoid useless abstractions around already usable
// embassy abstractions but to move some logic from initialization code
pub struct Sai {
    enabled: bool,
    sai: embassy_stm32::sai::Sai<'static, SAI1, u32>,
}

impl Sai {
    // FIXME: Know sys clock frequency!!!!
    pub fn cfg_i2s(sample_rate: u32, bit_depth: u8) -> sai::Config {
        assert!(
            sample_rate == 48_000 || sample_rate == 96_000,
            "Only these sample rates supported: 48kHz, 96kHz"
        );

        let mut sai_cfg = sai::Config::default();

        sai_cfg.mode = sai::Mode::Master;
        sai_cfg.tx_rx = sai::TxRx::Transmitter;
        sai_cfg.protocol = sai::Protocol::Free;
        sai_cfg.slot_size = sai::SlotSize::DataSize;
        sai_cfg.slot_count = sai::word::U4(2);
        sai_cfg.first_bit_offset = sai::word::U5(0);
        sai_cfg.slot_enable = 0b11;
        sai_cfg.data_size = match bit_depth {
            8 => sai::DataSize::Data8,
            10 => sai::DataSize::Data10,
            16 => sai::DataSize::Data16,
            20 => sai::DataSize::Data20,
            24 => sai::DataSize::Data24,
            32 => sai::DataSize::Data32,
            _ => panic!(),
        };
        // sai_cfg.data_size = sai::DataSize::Data32;
        sai_cfg.stereo_mono = sai::StereoMono::Stereo;
        sai_cfg.bit_order = sai::BitOrder::MsbFirst;
        sai_cfg.frame_sync_offset = sai::FrameSyncOffset::BeforeFirstBit;
        sai_cfg.frame_sync_polarity = sai::FrameSyncPolarity::ActiveLow;
        sai_cfg.frame_sync_active_level_length = sai::word::U7(bit_depth);
        // sai_cfg.frame_sync_active_level_length = sai::word::U7(32);
        sai_cfg.frame_sync_definition = sai::FrameSyncDefinition::ChannelIdentification;
        sai_cfg.frame_length = bit_depth * 2;
        // sai_cfg.frame_length = 64;
        sai_cfg.fifo_threshold = sai::FifoThreshold::Full;
        sai_cfg.clock_strobe = sai::ClockStrobe::Rising;
        // sai_cfg.master_clock_divider = match sample_rate {
        //     48_000 => sai::MasterClockDivider::Div50,
        //     96_000 => sai::MasterClockDivider::Div25,
        //     _ => panic!("Unsupported sample rate {}", sample_rate),
        // };
        sai_cfg.master_clock_divider = sai::MasterClockDivider::Div6;

        info!(
            "Make SAI Config: base clock frequency={}, bit_depth={}, sample_rate={}",
            embassy_stm32::rcc::frequency::<embassy_stm32::peripherals::SAI1>(),
            bit_depth,
            sample_rate,
        );

        sai_cfg
    }
}
