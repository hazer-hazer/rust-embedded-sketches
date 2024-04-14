use defmt::debug;

pub trait FromBeBytes<const SIZE: usize>: Sized {
    fn from_be_bytes(bytes: [u8; SIZE]) -> Self;
}

pub trait FromLeBytes<const SIZE: usize>: Sized {
    fn from_le_bytes(bytes: [u8; SIZE]) -> Self;
}

macro_rules! impl_from_e_bytes {
    ($($ty: ty),+) => {
        $(
            impl FromBeBytes<{ core::mem::size_of::<$ty>() }> for $ty {
                fn from_be_bytes(bytes: [u8; { core::mem::size_of::<$ty>() }]) -> Self {
                    Self::from_be_bytes(bytes)
                }
            }

            impl FromLeBytes<{ core::mem::size_of::<$ty>() }> for $ty {
                fn from_le_bytes(bytes: [u8; { core::mem::size_of::<$ty>() }]) -> Self {
                    Self::from_le_bytes(bytes)
                }
            }
        )+
    };
}

impl_from_e_bytes!(u16, u32, u64, i16, i32, i64);

pub struct FileReader<
    'a,
    const BUFFER_SIZE: usize,
    D,
    T,
    const MAX_DIRS: usize,
    const MAX_FILES: usize,
    const MAX_VOLUMES: usize,
> where
    D: embedded_sdmmc::BlockDevice,
    T: embedded_sdmmc::TimeSource,
{
    file: embedded_sdmmc::File<'a, D, T, MAX_DIRS, MAX_FILES, MAX_VOLUMES>,
    buffer: [u8; BUFFER_SIZE],
    pointer: usize,
    buffer_end: usize,
}

#[derive(Debug)]
pub enum ReadError<E>
where
    E: core::fmt::Debug,
{
    EOF,
    SdMMC(embedded_sdmmc::Error<E>),
}

impl<
        'a,
        D,
        T,
        const BUFFER_SIZE: usize,
        const MAX_DIRS: usize,
        const MAX_FILES: usize,
        const MAX_VOLUMES: usize,
    > FileReader<'a, BUFFER_SIZE, D, T, MAX_DIRS, MAX_FILES, MAX_VOLUMES>
where
    D: embedded_sdmmc::BlockDevice,
    T: embedded_sdmmc::TimeSource,
    D::Error: core::fmt::Debug,
{
    pub fn new(file: embedded_sdmmc::File<'a, D, T, MAX_DIRS, MAX_FILES, MAX_VOLUMES>) -> Self {
        Self {
            file,
            buffer: [0; BUFFER_SIZE],
            pointer: BUFFER_SIZE,
            buffer_end: BUFFER_SIZE,
        }
    }

    fn refetch(&mut self) -> Result<(), ReadError<D::Error>> {
        if self.pointer >= self.buffer.len() {
            self.pointer = 0;
            let read = self
                .file
                .read(&mut self.buffer)
                .map_err(|err| ReadError::SdMMC(err))?;
            self.buffer_end = read;
        }
        Ok(())
    }

    pub fn next_byte(&mut self) -> Result<u8, ReadError<D::Error>> {
        self.refetch()?;

        if self.pointer >= self.buffer_end {
            Err(ReadError::EOF)
        } else {
            let byte = self.buffer[self.pointer];
            self.pointer += 1;

            Ok(byte)
        }
    }

    // FIXME: Won't work properly if we already iterated last buffer, either change logic or offset file cursor
    pub fn next_buf(&mut self) -> Result<[u8; BUFFER_SIZE], ReadError<D::Error>> {
        self.refetch()?;

        self.pointer = BUFFER_SIZE;

        Ok(self.buffer)
    }

    pub fn pos(&self) -> u32 {
        self.file.offset() - (self.buffer.len() - self.pointer) as u32
    }

    pub fn eof(&self) -> bool {
        self.file.is_eof()
    }

    pub fn skip(&mut self, count: u32) -> Result<(), ReadError<D::Error>> {
        assert!(count <= i32::MAX as u32);

        debug!("Skip {} bytes after {}", count, self.pos());

        if (self.pointer + count as usize) < self.buffer.len() {
            // Offset fit into buffered data
            self.pointer += count as usize;
            Ok(())
        } else {
            // As cursor is at the point of the next buffer we need to offset cursor back
            let result = self
                .file
                .seek_from_start(self.pos() + count)
                .map_err(|err| ReadError::SdMMC(err))?;

            debug!(
                "Skipped {}, now file cursor at {}",
                count,
                self.file.offset()
            );

            // Invalidate the pointer, so next read will fetch next buffer
            self.pointer = BUFFER_SIZE;

            Ok(result)
        }
    }

    pub fn arr<const COUNT: usize>(&mut self) -> Result<[u8; COUNT], ReadError<D::Error>> {
        let mut bytes = [0; COUNT];
        for b in bytes.iter_mut() {
            *b = self.next_byte()?;
        }
        Ok(bytes)
    }

    pub fn next_le<const SIZE: usize, I: FromLeBytes<SIZE>>(
        &mut self,
    ) -> Result<I, ReadError<D::Error>> {
        self.arr().map(I::from_le_bytes)
    }

    pub fn next_be<const SIZE: usize, I: FromLeBytes<SIZE>>(
        &mut self,
    ) -> Result<I, ReadError<D::Error>> {
        self.arr().map(I::from_le_bytes)
    }
}
