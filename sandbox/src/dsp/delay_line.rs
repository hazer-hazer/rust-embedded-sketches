use alloc::vec::Vec;

pub struct DelayLine {
    buffer: Vec<f32>,
    pointer: usize,
}

impl DelayLine {
    pub fn new(length: usize) -> Self {
        Self {
            buffer: vec![0.0; length],
            pointer: 0,
        }
    }

    pub fn read(&self) -> f32 {
        self.buffer[self.pointer]
    }

    pub fn write_and_advance(&mut self, value: f32) {
        self.buffer[self.pointer] = value;

        if self.pointer == self.buffer.len() - 1 {
            self.pointer = 0
        } else {
            self.pointer += 1
        }
    }
}
