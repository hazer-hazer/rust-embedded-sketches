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

const BUFFER_SIZE: usize = 1024;
type Buffer = [u32; BUFFER_SIZE];
type BufferSingleton = &'static mut Buffer;

#[derive(Clone, Copy)]
pub enum LR {
    L,
    R,
}

pub struct PioI2S<P: PIOExt, DMA1: dma::ChannelIndex, DMA2: dma::ChannelIndex> {
    transfer: dma::double_buffer::Transfer<
        dma::Channel<DMA1>,
        dma::Channel<DMA2>,
        BufferSingleton,
        Tx<(P, SM1)>,
        dma::double_buffer::ReadNext<BufferSingleton>,
    >,
    buffer_queue: Queue<Buffer, 4>,
    buffer: Buffer,
    buffer_ptr: usize,
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
        // bit_depth: u8,
        data: Pin<D, P::PinFunction, PullNone>,
        bck: Pin<B, P::PinFunction, PullNone>,
        lrck: Pin<L, P::PinFunction, PullNone>,
    ) -> Self {
        // defmt::assert!(match bit_depth {
        //     16 | 24 | 32 => true,
        //     _ => false,
        // });

        let sys_clock = clocks.system_clock.freq().to_Hz();

        defmt::assert!(BUFFER_SIZE % 2 == 0, "Buffer size must be divisible by 2");
        defmt::assert!(lrck.id().num == bck.id().num + 1);
        defmt::assert!(
            (sys_clock as f32 / sample_rate as f32).fract() == 0.0,
            "System clock is not divisible by provided sample rate, jitter is not allowed"
        );

        debug!("Initializing I2S PIO");

        let (mut pio, _sm0, sm1, _, _) = pio.split(resets);

        let (clock_div_int, clock_div_frac) =
            get_clock_div(sys_clock as f32, sample_rate as f32 * 64.0);

        let i2s_master_out_prog =
            pio_proc::pio_file!("./src/rp2040_ext/i2s.pio", select_program("i2s_out_master"));

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
        transfer_cfg.pace(dma::Pace::PreferSink);
        let transfer = transfer_cfg.start().read_next(buf2);

        sm1.start();

        Self {
            transfer,
            buffer_queue: Queue::default(),
            buffer: [0; BUFFER_SIZE],
            buffer_ptr: 0,
        }
    }

    pub fn write(mut self, l: u32, r: u32) -> Self {
        self.buffer[self.buffer_ptr] = l;
        self.buffer[self.buffer_ptr + 1] = r;

        self.buffer_ptr += 2;
        if self.buffer_ptr >= self.buffer.len() {
            if let Err(_) = self.buffer_queue.enqueue(self.buffer) {
                error!("PIO I2S [WARN]: Buffer queue is full, failed to enqueue next buffer. The buffer is skipped!");
            }

            self.buffer = [0; BUFFER_SIZE];
            self.buffer_ptr = 0;
        }

        self
    }

    pub fn send(mut self) -> Self {
        if self.buffer_queue.is_empty() {
            // defmt::warn!("PIO I2S: Buffer queue is empty, nothing to send");
        } else if self.transfer.is_done() {
            let (next_buf, next_transfer) = self.transfer.wait();

            *next_buf = self.buffer_queue.dequeue().unwrap();

            self.transfer = next_transfer.read_next(next_buf);
        } else {
            defmt::debug!("PIO I2S: DMA Transfer is not complete");
        }

        self
    }
}
