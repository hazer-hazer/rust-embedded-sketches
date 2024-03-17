// //! This program creates an I2S interface via 3 seperate PIO state machines, toggling the
// //! GPIO 9, 10, and 11 pins (though 11 can be replaced with 25 to see it working
// //! on the led, just change the clock divider to something much closer to 65535 so
// //! you can see it).
// #![no_std]
// #![no_main]

// use cortex_m_rt::entry;
// use hal::gpio::{FunctionPio0, Pin};
// use hal::pac;
// use hal::pio::PIOExt;
// use hal::Sio;
// use panic_halt as _;
// use rp2040_hal as hal;

// #[link_section = ".boot2"]
// #[used]
// pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;

// #[entry]
// fn main() -> ! {
//     let mut pac = pac::Peripherals::take().unwrap();

//     let sio = Sio::new(pac.SIO);
//     let pins = hal::gpio::Pins::new(
//         pac.IO_BANK0,
//         pac.PADS_BANK0,
//         sio.gpio_bank0,
//         &mut pac.RESETS,
//     );

//     // configure pins for Pio
//     let _: Pin<_, FunctionPio0> = pins.gpio9.into_mode();
//     let _: Pin<_, FunctionPio0> = pins.gpio10.into_mode();
//     let _: Pin<_, FunctionPio0> = pins.gpio11.into_mode();
//     let _: Pin<_, FunctionPio0> = pins.gpio25.into_mode();

//     // PIN id for use inside of PIO
//     let pin9_i2s_data = 9;
//     let pin10_i2s_bck = 10;
//     let pin11_i2s_lrck = 11;
//     let pin25_led = 25;

//     // Define some simple PIO program.
//     let program_0 = pio_proc::pio_asm!(
//             "
//         pull block
//         out y, 32
//         mov x, y
//         main_loop:
//         set pins, 0 [5]
//         set pins, 1
//         irq clear 2
//         jmp x-- main_loop
//         irq clear 1
//         mov x, y
//         jmp main_loop
//             "
//     );

//     let program_1 = pio_proc::pio_asm!(
//             "
//     .wrap_target
//         set pins, 1
//         irq wait 1
//         set pins, 0
//         irq wait 1
//     .wrap
//             "
//     );

//     let program_1 = pio_proc::pio_asm!(
//         "
//     .wrap_target
//         set pins, 1
//         irq wait 1
//         set pins, 0
//         irq wait 1
//     .wrap
//         "
//     );

//     let program_2 = pio_proc::pio_asm!(
//         "
//         .wrap_target
//             set pins, 1 [4]
//             set pins, 0 [4]
//         .wrap
//         "
//     );

//     // Initialize and start PIO
//     let (mut pio, sm0, sm1, sm2, _) = pac.PIO0.split(&mut pac.RESETS);
//     /*
//     divisors assume a stock 130Mhz sys_clock
//     33.845 is the divisor for 384Khz
//     294.7 is the divisor for 44.1Khz
//     */
//     let div = 33.845 as f32;

//     // Defines the bit depth
//     let bit_accuracy = 32u32;

//     // install our PIO programs into the state machines and get a handle to the tx fifo on sm0.
//     let installed = pio.install(&program_0.program).unwrap();
//     let (mut sm0, _, mut tx) = rp2040_hal::pio::PIOBuilder::from_program(installed)
//         .set_pins(pin10_i2s_bck, 1)
//         .clock_divisor(div * 5.)
//         .build(sm0);
//     sm0.set_pindirs([(pin10_i2s_bck, hal::pio::PinDir::Output)]);

//     let installed = pio.install(&program_1.program).unwrap();
//     let (mut sm1, _, _) = rp2040_hal::pio::PIOBuilder::from_program(installed)
//         .set_pins(pin11_i2s_lrck, 1)
//         .clock_divisor(div)
//         .build(sm1);
//     sm1.set_pindirs([(pin11_i2s_lrck, hal::pio::PinDir::Output)]);

//     let installed = pio.install(&program_2.program).unwrap();
//     let (mut sm2, _, _) = rp2040_hal::pio::PIOBuilder::from_program(installed)
//         .set_pins(pin9_i2s_data, 1)
//         .clock_divisor(div)
//         .build(sm2);
//     sm1.set_pindirs([(pin9_i2s_data, hal::pio::PinDir::Output)]);

//     // Start both SMs at the same time
//     let group = sm0.with(sm1).with(sm2).sync().start();
//     cortex_m::asm::delay(10);
//     // write the bit_accuracy * 5 to the tx fifo. It is set at 5x due to the extra instructions per cycle that sm0 vs sm1 uses due to having
//     // to branch check and clear irq conditions for sm1, and keeps it in sync for the required timing.
//     tx.write(5u32 * bit_accuracy);

//     #[allow(clippy::empty_loop)]
//     loop {}
// }