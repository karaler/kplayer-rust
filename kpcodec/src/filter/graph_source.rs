use crate::filter::*;

pub trait KPGraphSourceRely {
    fn get_source(&self, media_type: &KPAVMediaType) -> Result<KPGraphSourceAttribute>;
}

pub enum KPGraphSourceAttribute {
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