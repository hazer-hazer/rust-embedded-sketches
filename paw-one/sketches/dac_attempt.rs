#![no_std]
#![no_main]

extern crate paw_one;

use cortex_m::singleton;
use defmt::*;
use {defmt_rtt as _, panic_probe as _};

use embassy_executor::Spawner;
use embassy_stm32::{
    dac::{self, DacCh1},
    pac::timer::vals::Mms,
    peripherals::{self, DMA1, DMA1_CH1},
    rcc::low_level::RccPeripheral,
    time::Hertz,
    timer::low_level::Basic16bitInstance,
};

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Program entered");

    let config = embassy_stm32::Config::default();

    info!("Configuring...");

    let p = embassy_stm32::init(config);

    let data = &calculate_array::<256>();

    peripherals::DMAMUX1::enable_and_reset();
    peripherals::DMA1::enable_and_reset();
    let mut dac = DacCh1::new(p.DAC1, p.DMA1_CH1, p.PA4);

    dac.set_triggering(true);
    dac.enable();

    loop {
        debug!("Send {}", data);
        dac.write(dac::ValueArray::Bit8(data), false).await;
    }
    // spawner.spawn(dac_task(p.TIM6, dac)).unwrap();
}

// #[embassy_executor::task]
// async fn dac_task(
//     mut tim: peripherals::TIM6,
//     mut dac: DacCh1<'static, peripherals::DAC1, peripherals::DMA1_CH1>,
// ) {
//     let data = &calculate_array::<256>();

//     info!("TIM6 frequency is {}", peripherals::TIM6::frequency());
//     const FREQUENCY: Hertz = Hertz::hz(200);

//     // Compute the reload value such that we obtain the FREQUENCY for the sine
//     let reload: u32 = (peripherals::TIM6::frequency().0 / FREQUENCY.0) / data.len() as u32;

//     // Depends on your clock and on the specific chip used, you may need higher or lower values here
//     if reload < 10 {
//         error!("Reload value {} below threshold!", reload);
//     }

//     dac.set_trigger(embassy_stm32::dac::TriggerSel::Tim6);
//     dac.set_triggering(true);
//     dac.enable();

//     peripherals::TIM6::enable_and_reset();
//     peripherals::TIM6::regs()
//         .arr()
//         .modify(|w| w.set_arr(reload as u16 - 1));
//     peripherals::TIM6::regs()
//         .cr2()
//         .modify(|w| w.set_mms(Mms::UPDATE));
//     peripherals::TIM6::regs().cr1().modify(|w| {
//         w.set_opm(false);
//         w.set_cen(true);
//     });

//     tim.enable_update_dma(true);

//     debug!(
//         "TIM6 Frequency {}, Target Frequency {}, Reload {}, Reload as u16 {}, Samples {}",
//         peripherals::TIM6::frequency(),
//         FREQUENCY,
//         reload,
//         reload as u16,
//         data.len()
//     );

//     // Loop technically not necessary if DMA circular mode is enabled
//     loop {
//         info!("Loop DAC1");
//         dac.write(dac::ValueArray::Bit8(data), true).await;
//     }
// }

fn to_sine_wave(v: u8) -> u8 {
    use micromath::F32Ext as _;
    if v >= 128 {
        // top half
        let r = 3.14 * ((v - 128) as f32 / 128.0);
        (r.sin() * 128.0 + 127.0) as u8
    } else {
        // bottom half
        let r = 3.14 + 3.14 * (v as f32 / 128.0);
        (r.sin() * 128.0 + 127.0) as u8
    }
}

fn calculate_array<const N: usize>() -> [u8; N] {
    let mut res = [0; N];
    let mut i = 0;
    while i < N {
        res[i] = to_sine_wave(i as u8);
        i += 1;
    }
    res
}
