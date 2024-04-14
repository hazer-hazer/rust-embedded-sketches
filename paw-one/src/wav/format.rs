#[derive(Debug, defmt::Format)]
pub enum WavAudioFormat {
    UNKNOWN,
    PCM,
    ADPCM,
    IEEE_FLOAT,
    VSELP,
    IBM_CVSD,
    ALAW,
    MULAW,
    DTS,
    DRM,
    OKI_ADPCM,
    DVI_ADPCM,
    MEDIASPACE_ADPCM,
    SIERRA_ADPCM,
    G723_ADPCM,
    DIGISTD,
    DIGIFIX,
    DIALOGIC_OKI_ADPCM,
    MEDIAVISION_ADPCM,
    CU_CODEC,
    YAMAHA_ADPCM,
    SONARC,
    DSPGROUP_TRUESPEECH,
    ECHOSC1,
    AUDIOFILE_AF36,
    APTX,
    AUDIOFILE_AF10,
    PROSODY_1612,
    LRC,
    DOLBY_AC2,
    GSM610,
    MSNAUDIO,
    ANTEX_ADPCME,
    CONTROL_RES_VQLPC,
    DIGIREAL,
    DIGIADPCM,
    CONTROL_RES_CR10,
    NMS_VBXADPCM,
    CS_IMAADPCM,
    ECHOSC3,
    ROCKWELL_ADPCM,
    ROCKWELL_DIGITALK,
    XEBEC,
    G721_ADPCM,
    G728_CELP,
    MSG723,
    MPEG,
    RT24,
    PAC,
    MPEGLAYER3,
    LUCENT_G723,
    CIRRUS,
    ESPCM,
    VOXWARE,
    CANOPUS_ATRAC,
    G726_ADPCM,
    G722_ADPCM,
    DSAT_DISPLAY,
    VOXWARE_BYTE_ALIGNED,
    VOXWARE_AC8,
    VOXWARE_AC10,
    VOXWARE_AC16,
    VOXWARE_AC20,
    VOXWARE_RT24,
    VOXWARE_RT29,
    VOXWARE_RT29HW,
    VOXWARE_VR12,
    VOXWARE_VR18,
    VOXWARE_TQ40,
    SOFTSOUND,
    VOXWARE_TQ60,
    MSRT24,
    G729A,
    MVI_MVI2,
    DF_G726,
    DF_GSM610,
    ISIAUDIO,
    ONLIVE,
    SBC24,
    DOLBY_AC3_SPDIF,
    MEDIASONIC_G723,
    PROSODY_8KBPS,
    ZYXEL_ADPCM,
    PHILIPS_LPCBB,
    PACKED,
    MALDEN_PHONYTALK,
    RHETOREX_ADPCM,
    IRAT,
    VIVO_G723,
    VIVO_SIREN,
    DIGITAL_G723,
    SANYO_LD_ADPCM,
    SIPROLAB_ACEPLNET,
    SIPROLAB_ACELP4800,
    SIPROLAB_ACELP8V3,
    SIPROLAB_G729,
    SIPROLAB_G729A,
    SIPROLAB_KELVIN,
    G726ADPCM,
    QUALCOMM_PUREVOICE,
    QUALCOMM_HALFRATE,
    TUBGSM,
    MSAUDIO1,
    UNISYS_NAP_ADPCM,
    UNISYS_NAP_ULAW,
    UNISYS_NAP_ALAW,
    UNISYS_NAP_16K,
    CREATIVE_ADPCM,
    CREATIVE_FASTSPEECH8,
    CREATIVE_FASTSPEECH10,
    UHER_ADPCM,
    QUARTERDECK,
    ILINK_VC,
    RAW_SPORT,
    ESST_AC3,
    IPI_HSX,
    IPI_RPELP,
    CS2,
    SONY_SCX,
    FM_TOWNS_SND,
    BTV_DIGITAL,
    QDESIGN_MUSIC,
    VME_VMPCM,
    TPC,
    OLIGSM,
    OLIADPCM,
    OLICELP,
    OLISBC,
    OLIOPR,
    LH_CODEC,
    NORRIS,
    SOUNDSPACE_MUSICOMPRESS,
    DVM,
}

impl From<u16> for WavAudioFormat {
    fn from(value: u16) -> Self {
        match value {
            0x0000 => Self::UNKNOWN,
            0x0001 => Self::PCM,
            0x0002 => Self::ADPCM,
            0x0003 => Self::IEEE_FLOAT,
            0x0004 => Self::VSELP,
            0x0005 => Self::IBM_CVSD,
            0x0006 => Self::ALAW,
            0x0007 => Self::MULAW,
            0x0008 => Self::DTS,
            0x0009 => Self::DRM,
            0x0010 => Self::OKI_ADPCM,
            0x0011 => Self::DVI_ADPCM,
            0x0012 => Self::MEDIASPACE_ADPCM,
            0x0013 => Self::SIERRA_ADPCM,
            0x0014 => Self::G723_ADPCM,
            0x0015 => Self::DIGISTD,
            0x0016 => Self::DIGIFIX,
            0x0017 => Self::DIALOGIC_OKI_ADPCM,
            0x0018 => Self::MEDIAVISION_ADPCM,
            0x0019 => Self::CU_CODEC,
            0x0020 => Self::YAMAHA_ADPCM,
            0x0021 => Self::SONARC,
            0x0022 => Self::DSPGROUP_TRUESPEECH,
            0x0023 => Self::ECHOSC1,
            0x0024 => Self::AUDIOFILE_AF36,
            0x0025 => Self::APTX,
            0x0026 => Self::AUDIOFILE_AF10,
            0x0027 => Self::PROSODY_1612,
            0x0028 => Self::LRC,
            0x0030 => Self::DOLBY_AC2,
            0x0031 => Self::GSM610,
            0x0032 => Self::MSNAUDIO,
            0x0033 => Self::ANTEX_ADPCME,
            0x0034 => Self::CONTROL_RES_VQLPC,
            0x0035 => Self::DIGIREAL,
            0x0036 => Self::DIGIADPCM,
            0x0037 => Self::CONTROL_RES_CR10,
            0x0038 => Self::NMS_VBXADPCM,
            0x0039 => Self::CS_IMAADPCM,
            0x003A => Self::ECHOSC3,
            0x003B => Self::ROCKWELL_ADPCM,
            0x003C => Self::ROCKWELL_DIGITALK,
            0x003D => Self::XEBEC,
            0x0040 => Self::G721_ADPCM,
            0x0041 => Self::G728_CELP,
            0x0042 => Self::MSG723,
            0x0050 => Self::MPEG,
            0x0052 => Self::RT24,
            0x0053 => Self::PAC,
            0x0055 => Self::MPEGLAYER3,
            0x0059 => Self::LUCENT_G723,
            0x0060 => Self::CIRRUS,
            0x0061 => Self::ESPCM,
            0x0062 => Self::VOXWARE,
            0x0063 => Self::CANOPUS_ATRAC,
            0x0064 => Self::G726_ADPCM,
            0x0065 => Self::G722_ADPCM,
            0x0067 => Self::DSAT_DISPLAY,
            0x0069 => Self::VOXWARE_BYTE_ALIGNED,
            0x0070 => Self::VOXWARE_AC8,
            0x0071 => Self::VOXWARE_AC10,
            0x0072 => Self::VOXWARE_AC16,
            0x0073 => Self::VOXWARE_AC20,
            0x0074 => Self::VOXWARE_RT24,
            0x0075 => Self::VOXWARE_RT29,
            0x0076 => Self::VOXWARE_RT29HW,
            0x0077 => Self::VOXWARE_VR12,
            0x0078 => Self::VOXWARE_VR18,
            0x0079 => Self::VOXWARE_TQ40,
            0x0080 => Self::SOFTSOUND,
            0x0081 => Self::VOXWARE_TQ60,
            0x0082 => Self::MSRT24,
            0x0083 => Self::G729A,
            0x0084 => Self::MVI_MVI2,
            0x0085 => Self::DF_G726,
            0x0086 => Self::DF_GSM610,
            0x0088 => Self::ISIAUDIO,
            0x0089 => Self::ONLIVE,
            0x0091 => Self::SBC24,
            0x0092 => Self::DOLBY_AC3_SPDIF,
            0x0093 => Self::MEDIASONIC_G723,
            0x0094 => Self::PROSODY_8KBPS,
            0x0097 => Self::ZYXEL_ADPCM,
            0x0098 => Self::PHILIPS_LPCBB,
            0x0099 => Self::PACKED,
            0x00A0 => Self::MALDEN_PHONYTALK,
            0x0100 => Self::RHETOREX_ADPCM,
            0x0101 => Self::IRAT,
            0x0111 => Self::VIVO_G723,
            0x0112 => Self::VIVO_SIREN,
            0x0123 => Self::DIGITAL_G723,
            0x0125 => Self::SANYO_LD_ADPCM,
            0x0130 => Self::SIPROLAB_ACEPLNET,
            0x0131 => Self::SIPROLAB_ACELP4800,
            0x0132 => Self::SIPROLAB_ACELP8V3,
            0x0133 => Self::SIPROLAB_G729,
            0x0134 => Self::SIPROLAB_G729A,
            0x0135 => Self::SIPROLAB_KELVIN,
            0x0140 => Self::G726ADPCM,
            0x0150 => Self::QUALCOMM_PUREVOICE,
            0x0151 => Self::QUALCOMM_HALFRATE,
            0x0155 => Self::TUBGSM,
            0x0160 => Self::MSAUDIO1,
            0x0170 => Self::UNISYS_NAP_ADPCM,
            0x0171 => Self::UNISYS_NAP_ULAW,
            0x0172 => Self::UNISYS_NAP_ALAW,
            0x0173 => Self::UNISYS_NAP_16K,
            0x0200 => Self::CREATIVE_ADPCM,
            0x0202 => Self::CREATIVE_FASTSPEECH8,
            0x0203 => Self::CREATIVE_FASTSPEECH10,
            0x0210 => Self::UHER_ADPCM,
            0x0220 => Self::QUARTERDECK,
            0x0230 => Self::ILINK_VC,
            0x0240 => Self::RAW_SPORT,
            0x0241 => Self::ESST_AC3,
            0x0250 => Self::IPI_HSX,
            0x0251 => Self::IPI_RPELP,
            0x0260 => Self::CS2,
            0x0270 => Self::SONY_SCX,
            0x0300 => Self::FM_TOWNS_SND,
            0x0400 => Self::BTV_DIGITAL,
            0x0450 => Self::QDESIGN_MUSIC,
            0x0680 => Self::VME_VMPCM,
            0x0681 => Self::TPC,
            0x1000 => Self::OLIGSM,
            0x1001 => Self::OLIADPCM,
            0x1002 => Self::OLICELP,
            0x1003 => Self::OLISBC,
            0x1004 => Self::OLIOPR,
            0x1100 => Self::LH_CODEC,
            0x1400 => Self::NORRIS,
            0x1500 => Self::SOUNDSPACE_MUSICOMPRESS,
            0x2000 => Self::DVM,
            _ => panic!("Unknown WAV audio format"),
        }
    }
}