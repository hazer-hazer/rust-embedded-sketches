pub struct FmtBuf<const SIZE: usize = 64> {
    buf: [u8; SIZE],
    ptr: usize,
}

impl<const SIZE: usize> FmtBuf<SIZE> {
    pub fn new() -> Self {
        Self {
            buf: [0; SIZE],
            ptr: 0,
        }
    }

    pub fn reset(&mut self) {
        self.ptr = 0
    }

    pub fn as_str(&self) -> &str {
        core::str::from_utf8(&self.buf[0..self.ptr]).unwrap()
    }

    pub fn remaining(&self) -> usize {
        self.buf.len() - self.ptr
    }

    pub fn write(&mut self, s: &str) {
        // Clip input, in case with overflow of content the buffer length will be used
        let len = core::cmp::min(s.len(), self.remaining());

        // Copy contents into passed string
        self.buf[self.ptr..(self.ptr + len)].copy_from_slice(&s.as_bytes()[0..len]);

        self.ptr += len;
    }
}

impl<const SIZE: usize> core::fmt::Write for FmtBuf<SIZE> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.write(s);
        Ok(())
    }
}
