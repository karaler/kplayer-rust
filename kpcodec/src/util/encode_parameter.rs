use crate::util::*;

#[derive(Debug, Display, EnumString)]
pub enum KPEncodeParameterProfile {
    #[strum(serialize = "high")]
    High,
}

#[derive(Debug, Display, EnumString)]
pub enum KPEncodeParameterPreset {
    #[strum(serialize = "veryfast")]
    VeryFast,
}

#[derive(Debug)]
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
}