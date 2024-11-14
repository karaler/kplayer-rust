use crate::decode::*;
use std::collections::HashMap;
use crate::decode::decode::{KPDecode, KPDecodeIterator};
use crate::filter::graph_source::{KPGraphSourceAttribute, KPGraphSourceRely};
use crate::util::alias::KPAVMediaType;

pub struct KPMixCodec {
    source: HashMap<KPAVMediaType, KPDecode>,
    maintainer_source: Option<KPAVMediaType>,

    // options
    expect_stream_index: HashMap<KPAVMediaType, Option<usize>>,
    encode_hardware: bool,

    // state
    status: KPCodecStatus,
}

pub struct KPMixCodecIterator<'a> {
    mix: &'a mut KPMixCodec,
}

impl<'a> Iterator for KPMixCodecIterator<'a> {
    type Item = Result<(KPAVMediaType, KPAVFrame)>;

    fn next(&mut self) -> Option<Self::Item> {
        let mix = &mut self.mix;

        loop {
            for (_, decode) in mix.source.iter() {
                if decode.status == KPCodecStatus::Ended {
                    mix.status = KPCodecStatus::Ended;
                    return None;
                }
            }
            let video_position = mix.source.get_mut(&KPAVMediaType::KPAVMEDIA_TYPE_VIDEO).unwrap().position;
            let audio_position = mix.source.get_mut(&KPAVMediaType::KPAVMEDIA_TYPE_AUDIO).unwrap().position;

            let mut decode;
            if video_position <= audio_position {
                decode = mix.source.get_mut(&KPAVMediaType::KPAVMEDIA_TYPE_VIDEO).unwrap();
            } else {
                decode = mix.source.get_mut(&KPAVMediaType::KPAVMEDIA_TYPE_AUDIO).unwrap();
            }
            if let Err(err) = decode.stream_to_codec() { return Some(Err(err)); }
            return match decode.stream_from_codec() {
                Ok(f) => match f {
                    None => continue,
                    Some(frame) => Some(Ok(frame))
                },
                Err(err) => Some(Err(err))
            };
        }
    }
}

impl KPGraphSourceRely for KPMixCodec {
    fn get_source(&self, media_type: &KPAVMediaType) -> Result<KPGraphSourceAttribute> {
        assert_eq!(self.status, KPCodecStatus::Started);
        self.source.get(media_type).unwrap().get_source(media_type)
    }
}

impl KPMixCodec {
    pub fn iter(&mut self) -> KPMixCodecIterator {
        KPMixCodecIterator {
            mix: self,
        }
    }
}

impl KPMixCodec {
    pub fn new<T: ToString>(video_path: T, audio_path: T, maintainer_source: Option<KPAVMediaType>) -> Self {
        // create decode
        let video_decode = KPDecode::new(video_path);
        let audio_decode = KPDecode::new(audio_path);
        let mut source = HashMap::new();
        source.insert(KPAVMediaType::KPAVMEDIA_TYPE_VIDEO, video_decode);
        source.insert(KPAVMediaType::KPAVMEDIA_TYPE_AUDIO, audio_decode);

        KPMixCodec {
            source,
            maintainer_source,
            expect_stream_index: HashMap::new(),
            encode_hardware: false,
            status: Default::default(),
        }
    }

    // set flag
    pub fn set_expect_stream(&mut self, expect_streams: HashMap<KPAVMediaType, Option<usize>>) -> &mut Self {
        self.expect_stream_index = expect_streams;
        self
    }

    pub fn open(&mut self) -> Result<()> {
        for (media_type, decode) in self.source.iter_mut() {
            // open file
            decode.open()?;

            // set expect stream
            if let Some(expect_stream) = self.expect_stream_index.get(media_type) {
                let mut expect_streams = HashMap::new();
                expect_streams.insert(media_type.clone(), expect_stream.clone());
                decode.set_expect_stream(expect_streams);
            }

            // set maintainer loop
            if let Some(maintainer_media_type) = self.maintainer_source {
                if maintainer_media_type.ne(media_type) {
                    decode.set_enable_loop(true);
                }
            }
        }

        self.status = KPCodecStatus::Opened;
        Ok(())
    }

    pub fn find_streams(&mut self) -> Result<()> {
        for (_, decode) in self.source.iter_mut() {
            decode.find_streams()?;
        }
        Ok(())
    }

    pub fn open_codec(&mut self) -> Result<()> {
        for (_, decode) in self.source.iter_mut() {
            decode.open_codec()?;
        }

        self.status = KPCodecStatus::Started;
        Ok(())
    }

    pub fn get_status(&self) -> &KPCodecStatus {
        &self.status
    }
}

#[test]
fn test_mix() -> Result<()> {
    initialize();

    let mut mix = KPMixCodec::new(env::var("INPUT_SHORT_PATH")?, env::var("INPUT_SHORT_PATH")?, None);
    mix.open()?;
    mix.find_streams()?;
    mix.open_codec()?;

    for frame in mix.iter() {
        let (media_type, get_frame) = frame?;
        info!("media type: {}, frame: {:?}", media_type, get_frame);
    }
    Ok(())
}