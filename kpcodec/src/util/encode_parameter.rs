use crate::util::*;

#[derive(Debug, Display, EnumString, Clone)]
pub enum KPEncodeParameterProfile {
    #[strum(serialize = "high")]
    High,
}

#[derive(Debug, Display, EnumString, Clone)]
pub enum KPEncodeParameterPreset {
    #[strum(serialize = "veryfast")]
    VeryFast,
}

#[derive(Debug, Clone)]
pub enum KPEncodeParameter {
    Video {
        codec_id: KPAVCodecId,
        width: usize,
        height: usize,
        pix_fmt: KPAVPixelFormat,
        framerate: KPAVRational,
        max_bitrate: usize,
        quality: u16,
        profile: KPEncodeParameterProfile,
        preset: KPEncodeParameterPreset,
        gop_uint: u16,
        metadata: BTreeMap<String, String>,
    },
    Audio {
        codec_id: KPAVCodecId,
        sample_rate: usize,
        sample_fmt: KPAVSampleFormat,
        channel_layout: usize,
        channels: usize,
        metadata: BTreeMap<String, String>,
    },
}

impl KPEncodeParameter {
    pub fn default(media_type: &KPAVMediaType) -> KPEncodeParameter {
        match media_type {
            m if m.eq(&KPAVMediaType::KPAVMEDIA_TYPE_VIDEO) => {
                KPEncodeParameter::Video {
                    codec_id: KPAVCodecId::from(AV_CODEC_ID_H264),
                    width: 848,
                    height: 480,
                    pix_fmt: KPAVPixelFormat::from(AV_PIX_FMT_YUV420P),
                    max_bitrate: 0,
                    quality: 0,
                    profile: KPEncodeParameterProfile::High,
                    preset: KPEncodeParameterPreset::VeryFast,
                    framerate: KPAVRational::from_fps(29),
                    gop_uint: 2,
                    metadata: BTreeMap::new(),
                }
            }
            m if m.eq(&KPAVMediaType::KPAVMEDIA_TYPE_AUDIO) => {
                KPEncodeParameter::Audio {
                    codec_id: KPAVCodecId::from(AV_CODEC_ID_AAC),
                    sample_rate: 48000,
                    sample_fmt: KPAVSampleFormat::from(AV_SAMPLE_FMT_FLTP),
                    channel_layout: 3,
                    channels: 2,
                    metadata: BTreeMap::new(),
                }
            }
            _ => {
                panic!("not support media type");
            }
        }
    }

    pub fn get_video_parameter(&self) -> Result<(KPAVCodecId, usize, usize, KPAVPixelFormat, KPAVRational, usize, u16, KPEncodeParameterProfile, KPEncodeParameterPreset, u16, BTreeMap<String, String>)> {
        match self {
            KPEncodeParameter::Video { codec_id, width, height, pix_fmt, framerate, max_bitrate, quality, profile, preset, gop_uint, metadata } => {
                Ok((codec_id.clone(), width.clone(), height.clone(), pix_fmt.clone(), framerate.clone(), max_bitrate.clone(), quality.clone(), profile.clone(), preset.clone(), gop_uint.clone(), metadata.clone()))
            }
            KPEncodeParameter::Audio { .. } => {
                Err(anyhow!("not support audio parameter"))
            }
        }
    }

    pub fn get_audio_parameter(&self) -> Result<(KPAVCodecId, usize, KPAVSampleFormat, usize, usize, BTreeMap<String, String>)> {
        match self {
            KPEncodeParameter::Audio { codec_id, sample_rate, sample_fmt, channel_layout, channels, metadata } => {
                Ok((codec_id.clone(), sample_rate.clone(), sample_fmt.clone(), channel_layout.clone(), channels.clone(), metadata.clone()))
            }
            KPEncodeParameter::Video { .. } => {
                Err(anyhow!("not support video parameter"))
            }
        }
    }
}