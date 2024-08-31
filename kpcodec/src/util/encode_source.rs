use crate::util::*;
use crate::util::encode_parameter::KPEncodeParameter;

pub trait KPEncodeSourceRely {
    fn get_source(&self, media_type: &KPAVMediaType) -> Result<KPEncodeSourceAttribute>;
}

pub enum KPEncodeSourceAttribute {
    Video {
        width: usize,
        height: usize,
        pix_fmt: KPAVPixelFormat,
        time_base: KPAVRational,
        frame_rate: KPAVRational,
        pixel_aspect: KPAVRational,
    },
    Audio {
        sample_rate: usize,
        sample_fmt: KPAVSampleFormat,
        channel_layout: usize,
        channels: usize,
        time_base: KPAVRational,
    },
}