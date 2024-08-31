use crate::util::*;

pub trait KPEncodeParameterRely {
    fn get_parameter(&self, media_type: &KPAVMediaType) -> Result<KPEncodeParameter>;
}

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
        max_bitrate: usize,
        quality: u16,
        profile: KPEncodeParameterProfile,
        preset: KPEncodeParameterPreset,
        fps: KPAVRational,
        gop_uint: u16,
        metadata: BTreeMap<String, String>,
    },
    Audio {
        codec_id: KPAVCodecId,
        metadata: BTreeMap<String, String>,
    },
}

impl KPEncodeParameter {
    pub fn default(media_type: &KPAVMediaType) -> KPEncodeParameter {
        match media_type {
            m if m.eq(&KPAVMediaType::KPAVMEDIA_TYPE_VIDEO) => {
                KPEncodeParameter::Video {
                    codec_id: KPAVCodecId::from(AV_CODEC_ID_H264),
                    max_bitrate: 0,
                    quality: 0,
                    profile: KPEncodeParameterProfile::High,
                    preset: KPEncodeParameterPreset::VeryFast,
                    fps: KPAVRational::from_fps(25),
                    gop_uint: 2,
                    metadata: BTreeMap::new(),
                }
            }
            m if m.eq(&KPAVMediaType::KPAVMEDIA_TYPE_AUDIO) => {
                KPEncodeParameter::Audio {
                    codec_id: KPAVCodecId::from(AV_CODEC_ID_AAC),
                    metadata: BTreeMap::new(),
                }
            }
            _ => {
                panic!("not support media type");
            }
        }
    }
}