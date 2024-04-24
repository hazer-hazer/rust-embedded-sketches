use defmt::debug;

use crate::sd::{FileReader, ReadError};

use self::format::WavAudioFormat;

pub mod format;

#[derive(Debug, defmt::Format)]
pub struct WavHeader {
    pub chunk_id: [u8; 4],
    pub chunk_size: u32,
    pub format: [u8; 4],
    // subchunk1_id: [u8; 4],
    // subchunk1_size: u32,
    pub audio_format: WavAudioFormat,
    pub num_channels: u16,
    pub sample_rate: u32,
    pub byte_rate: u32,
    pub block_align: u16,
    pub bits_per_sample: u16,
    // subchunk2_id: [u8; 4],
    // subchunk2_size: u32,
    pub data_size: u32,
}

#[derive(Debug)]
pub enum WavHeaderError<E>
where
    E: core::fmt::Debug,
{
    SdMmc(E),
    ReadError(ReadError<E>),
    NoDataSubchunk,
}

impl<E: core::fmt::Debug> From<ReadError<E>> for WavHeaderError<E> {
    fn from(value: ReadError<E>) -> Self {
        Self::ReadError(value)
    }
}

impl WavHeader {
    pub fn by_file_reader<
        'a,
        const BUFFER_SIZE: usize,
        D,
        T,
        const MAX_DIRS: usize,
        const MAX_FILES: usize,
        const MAX_VOLUMES: usize,
    >(
        reader: &mut FileReader<'a, BUFFER_SIZE, D, T, MAX_DIRS, MAX_FILES, MAX_VOLUMES>,
    ) -> Result<Self, WavHeaderError<D::Error>>
    where
        D: embedded_sdmmc::BlockDevice,
        T: embedded_sdmmc::TimeSource,
        D::Error: core::fmt::Debug,
    {
        let chunk_id = reader.arr()?;
        defmt::assert_eq!(
            chunk_id.as_slice(),
            b"RIFF",
            "WAV Header invalid chunk ID: {:a} (must be RIFF)",
            chunk_id
        );

        let chunk_size = reader.next_le()?;

        let format = reader.arr()?;
        defmt::assert_eq!(
            format.as_slice(),
            b"WAVE",
            "WAV Header invalid format {:a} (must be WAVE)",
            format
        );

        let fmt_subchunk_id = reader.arr::<4>()?;
        defmt::assert_eq!(
            fmt_subchunk_id.as_slice(),
            b"fmt ",
            "WAV Header invalid subchunk1 id {:a} (must be \"fmt \"",
            fmt_subchunk_id
        );

        const KNOWN_FMT_BYTES_COUNT: u32 = 16;

        let fmt_size: u32 = reader.next_le()?;
        defmt::assert!(fmt_size >= KNOWN_FMT_BYTES_COUNT);

        let audio_format: u16 = reader.next_le()?;

        let num_channels = reader.next_le()?;

        let sample_rate = reader.next_le()?;

        let byte_rate = reader.next_le()?;
        let block_align = reader.next_le()?;
        let bits_per_sample = reader.next_le()?;

        let skip_unknown_fmt = fmt_size - KNOWN_FMT_BYTES_COUNT;
        if skip_unknown_fmt > 0 {
            debug!("Skip unknown fmt header part of {} bytes", skip_unknown_fmt);
            reader.skip(skip_unknown_fmt)?;
        }

        let data_size = {
            let mut data_size = None;
            while !reader.eof() {
                let subchunk_id = reader.arr::<4>()?;
                let subchunk_size = reader.next_le()?;
                if subchunk_id.as_slice() == b"data" {
                    data_size = Some(subchunk_size);
                    break;
                }
                debug!(
                    "Skip unknown subchunk {:a} of {} bytes",
                    subchunk_id.as_slice(),
                    subchunk_size
                );
                reader.skip(subchunk_size)?;
            }
            data_size.ok_or(WavHeaderError::NoDataSubchunk)?
        };

        Ok(Self {
            chunk_id,
            chunk_size,
            format: format.into(),
            // subchunk1_id,
            // subchunk1_size,
            audio_format: audio_format.into(),
            num_channels,
            sample_rate,
            byte_rate,
            block_align,
            bits_per_sample,
            // subchunk2_id,
            // subchunk2_size,
            data_size,
        })
    }
}
