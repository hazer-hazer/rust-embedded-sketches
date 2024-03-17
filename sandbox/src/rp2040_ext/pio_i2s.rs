use crate::bsp::hal::{
    clocks::ClocksManager,
    dma,
    gpio::{FunctionPio0, Pin, PinId, PullNone},
    pac::{Peripherals, PIO0, RESETS},
    pio::{Buffers, PIOBuilder, PIOExt, PinDir, ShiftDirection, StateMachineIndex, Tx, SM1},
    Clock,
};
use cortex_m::singleton;
use defmt::{debug, error};
use heapless::spsc::Queue;
use num_traits::Float;
use pio_proc::pio_asm;

fn get_clock_div(num: f32, denom: f32) -> (u16, u8) {
    let clock_div = num / denom;
    (clock_div as u16, (clock_div.fract() * u8::MAX as f32) as u8)
}

const BUFFER_SIZE: usize = 16;
const QUEUE_SIZE: usize = BUFFER_SIZE * 2;
type Buffer = &'static mut [u32; BUFFER_SIZE];

#[derive(Clone, Copy)]
pub enum LR {
    L,
    R,
}

pub struct PioI2S<P: PIOExt, DMA1: dma::ChannelIndex, DMA2: dma::ChannelIndex> {
    transfer: dma::double_buffer::Transfer<
        dma::Channel<DMA1>,
        dma::Channel<DMA2>,
        Buffer,
        Tx<(P, SM1)>,
        dma::double_buffer::ReadNext<Buffer>,
    >,
    l_queue: Queue<f32, QUEUE_SIZE>,
    r_queue: Queue<f32, QUEUE_SIZE>,
}

impl<P: PIOExt, DMA1: dma::ChannelIndex, DMA2: dma::ChannelIndex> PioI2S<P, DMA1, DMA2> {
    pub fn new<
        // S: PinId,
        D: PinId,
        B: PinId,
        L: PinId,
    >(
        pio: P,
        dma_buffers: (dma::Channel<DMA1>, dma::Channel<DMA2>),
        resets: &mut RESETS,
        sample_rate: u32,
        clocks: &ClocksManager,
        // sck: Pin<S, P::PinFunction, PullNone>,
        data: Pin<D, P::PinFunction, PullNone>,
        bck: Pin<B, P::PinFunction, PullNone>,
        lrck: Pin<L, P::PinFunction, PullNone>,
    ) -> Self {
        defmt::assert!(lrck.id().num == bck.id().num + 1);

        debug!("Initializing I2S PIO");

        let sys_clock = clocks.system_clock.freq().to_Hz();
        let (mut pio, _sm0, sm1, _, _) = pio.split(resets);

        let (clock_div_int, clock_div_frac) =
            get_clock_div(sys_clock as f32, sample_rate as f32 * 64.0 / 2.0);

        // let pio_prog = pio_proc::pio_asm!(
        //     "
        //     pull block
        //     out y, 32
        //     mov x, y
        //     main_loop:
        //     set pins, 0 [5]
        //     set pins, 1
        //     irq clear 2
        //     jmp x-- main_loop
        //     irq clear 1
        //     mov x, y
        //     jmp main_loop
        //         "
        // );

        // let installed = pio.install(&pio_prog.program).unwrap();
        // let (mut sm0, _, mut tx) = PIOBuilder::from_installed_program(installed)
        //     .set_pins(bck.id().num, 1)
        //     .clock_divisor_fixed_point(clock_div_int * 5, clock_div_frac)
        //     .build(sm0);
        // sm0.set_pindirs([(bck.id().num, PinDir::Output)]);

        // let pio_prog = pio_proc::pio_asm!(
        //     "
        // .wrap_target
        //     set pins, 1
        //     irq wait 1
        //     set pins, 0
        //     irq wait 1
        // .wrap
        //         "
        // );

        // let installed = pio.install(&pio_prog.program).unwrap();
        // let (mut sm1, _, _) = PIOBuilder::from_installed_program(installed)
        //     .set_pins(lrck.id().num, 1)
        //     .clock_divisor_fixed_point(clock_div_int, clock_div_frac)
        //     .build(sm1);
        // sm1.set_pindirs([(lrck.id().num, PinDir::Output)]);

        // let pio_prog = pio_proc::pio_asm!(
        //     "
        //     .wrap_target
        //         set pins, 1 [4]
        //         set pins, 0 [4]
        //     .wrap
        //     "
        // );

        // let installed = pio.install(&pio_prog.program).unwrap();
        // let (mut sm2, _, _) = PIOBuilder::from_installed_program(installed)
        //     .set_pins(data.id().num, 1)
        //     .clock_divisor_fixed_point(clock_div_int, clock_div_frac)
        //     .build(sm2);
        // sm2.set_pindirs([(data.id().num, PinDir::Output)]);

        // let group = sm0.with(sm1).with

        // let i2s_master_clock_prog = pio_proc::pio_file!(
        //     "./src/rp2040_ext/i2s.pio",
        //     select_program("i2s_master_clock")
        // );

        // let (mut sm0, _, _) =
        //     PIOBuilder::from_program(pio.install(&i2s_master_clock_prog.program).unwrap())
        //         .set_pins(sck.id().num, 1)
        //         .build(sm0);

        // sm0.set_pindirs([(sck.id().num, PinDir::Output)]);

        let i2s_master_out_prog =
            pio_proc::pio_file!("./src/rp2040_ext/i2s.pio", select_program("i2s_out_master"));

        // let (mut sm0, _, _) = PIOBuilder::from_installed_program(pio.install(i2s_master_clock_prog.program).unwrap()).set_pins(base, count)

        debug!("Run PIO I2S on BCK={}, LRCK={}, DIN={}.\nSample rate: {}\nbit_depth: 32\nclock_div (for sys {}): {} + ({}/256)", bck.id().num, lrck.id().num, data.id().num, sample_rate, sys_clock, clock_div_int, clock_div_frac);

        let (mut sm1, _, tx) =
            PIOBuilder::from_program(pio.install(&i2s_master_out_prog.program).unwrap())
                .out_pins(data.id().num, 1)
                .side_set_pin_base(bck.id().num)
                .pull_threshold(32)
                .out_shift_direction(ShiftDirection::Left)
                .autopull(true)
                .buffers(Buffers::OnlyTx)
                .clock_divisor_fixed_point(clock_div_int, clock_div_frac)
                .build(sm1);

        sm1.set_pindirs([
            (data.id().num, PinDir::Output),
            (lrck.id().num, PinDir::Output),
            (bck.id().num, PinDir::Output),
        ]);

        let buf1 = singleton!(: [u32; BUFFER_SIZE] = [0; BUFFER_SIZE]).unwrap();
        let buf2 = singleton!(: [u32; BUFFER_SIZE] = [0; BUFFER_SIZE]).unwrap();
        let mut transfer_cfg = dma::double_buffer::Config::new(dma_buffers, buf1, tx);
        // transfer_cfg.pace(dma::Pace::PreferSink);
        let transfer = transfer_cfg.start().read_next(buf2);

        let mut l_queue = Queue::new();
        let mut r_queue = Queue::new();
        for _ in 0..BUFFER_SIZE {
            l_queue.enqueue(0.0).unwrap();
            r_queue.enqueue(0.0).unwrap();
        }

        sm1.start();

        Self {
            transfer,
            l_queue,
            r_queue,
        }
    }

    pub fn write_both(self, word: f32) -> Self {
        self.write(LR::L, word).write(LR::R, word)
    }

    pub fn write(mut self, lr: LR, word: f32) -> Self {
        let sample_queue = match lr {
            LR::L => &mut self.l_queue,
            LR::R => &mut self.r_queue,
        };

        if let Err(_) = sample_queue.enqueue(word) {
            error!("PIO I2S [WARN]: Sample queue is full, failed to enqueue next value. The value is skipped!");
        }

        self
    }

    pub fn maybe_send(self) -> Self {
        if self.l_queue.len() >= BUFFER_SIZE || self.r_queue.len() >= BUFFER_SIZE {
            self.send()
        } else {
            self
        }
    }

    pub fn send(mut self) -> Self {
        if self.transfer.is_done() {
            let (next_buf, next_transfer) = self.transfer.wait();

            defmt::info!(
                "PIO I2S [Info]: Completed transfer of {} samples; {} non-zero samples",
                next_buf.len(),
                next_buf
                    .iter()
                    .fold(0, |acc, s| if *s != 0 { acc + 1 } else { acc }),
            );
            defmt::debug!("PIO I2S: Sent buffer:\n{}", next_buf);

            for (i, sample) in next_buf.iter_mut().enumerate() {
                let s = if i % 2 == 0 {
                    self.l_queue.dequeue()
                } else {
                    self.r_queue.dequeue()
                };
                // .unwrap_or(0.0);
                let s = if let Some(s) = s {
                    s
                } else {
                    defmt::debug!("Queue is empty, producing 0");
                    0.0
                };
                // let s = (s * 2_147_483_648.0) as i32;
                // let s = if s < 0 {
                //     (s + 2_147_483_647 + 1) as u32
                // } else {
                //     s as u32 + 2_147_483_648
                // };
                *sample = (s * i32::MAX as f32) as i32 as u32;
            }

            self.transfer = next_transfer.read_next(next_buf);
        }

        self
    }
}
