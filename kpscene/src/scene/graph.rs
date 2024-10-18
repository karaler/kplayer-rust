use std::fs;
use rusty_ffmpeg::ffi::{AV_SAMPLE_FMT_FLT, AV_SAMPLE_FMT_FLTP};
use kpcodec::util::codec_status::KPCodecStatus;
use crate::scene::*;
use crate::scene::engine::wasm::KPEngine;

pub trait KPSceneGraph {
    fn add_scene(&mut self, scene: &KPScene) -> Result<()>;
}

impl KPSceneGraph for KPGraph {
    fn add_scene(&mut self, scene: &KPScene) -> Result<()> {
        for get_filter in scene.get_filters() {
            self.add_filter(get_filter)?;
        }
        Ok(())
    }
}

#[tokio::test]
async fn main() -> Result<()> {
    initialize();

    let mut decode = KPDecode::new(env::var("INPUT_PATH").unwrap());
    decode.open().unwrap();

    // set expect stream
    let mut expect_streams = HashMap::new();
    expect_streams.insert(KPAVMediaType::KPAVMEDIA_TYPE_VIDEO, None);
    expect_streams.insert(KPAVMediaType::KPAVMEDIA_TYPE_AUDIO, None);
    decode.set_expect_stream(expect_streams.clone());
    decode.find_streams().unwrap();
    decode.open_codec().unwrap();

    // create encode custom parameters
    let mut encode_parameter = BTreeMap::new();
    for (media_type, _) in expect_streams.iter() {
        if media_type.eq(&KPAVMediaType::KPAVMEDIA_TYPE_VIDEO) {
            encode_parameter.insert(media_type.clone(), KPEncodeParameter::default(&KPAVMediaType::KPAVMEDIA_TYPE_VIDEO));
        } else if media_type.eq(&KPAVMediaType::KPAVMEDIA_TYPE_AUDIO) {
            encode_parameter.insert(media_type.clone(), KPEncodeParameter::default(&KPAVMediaType::KPAVMEDIA_TYPE_AUDIO));
        }
    }

    // load plugin
    let wasm_path = env::var("TEXT_WASM_PATH").unwrap();
    let file_data = fs::read(wasm_path)?;
    let engine = KPEngine::new(file_data).await?;
    let scene = KPScene::from_engine(&engine);

    // create graph
    let mut graph_map = HashMap::new();
    for (media_type, _) in expect_streams.iter() {
        let mut graph = KPGraph::new(media_type);
        graph.injection_source(&decode).unwrap();

        if media_type.eq(&KPAVMediaType::KPAVMEDIA_TYPE_VIDEO) {
            {
                let mut argument = BTreeMap::new();
                argument.insert("w".to_string(), "848".to_string());
                argument.insert("h".to_string(), "480".to_string());
                let filter = KPFilter::new("scale", argument, vec![]).unwrap();
                graph.add_filter(vec![filter]).unwrap();
            }
            {
                let mut argument = BTreeMap::new();
                argument.insert("pix_fmts".to_string(), KPAVPixelFormat::from(AV_PIX_FMT_YUV420P).to_string());
                let filter = KPFilter::new("format", argument, vec![]).unwrap();
                graph.add_filter(vec![filter]).unwrap();
            }
            {
                let mut argument = BTreeMap::new();
                argument.insert("fps".to_string(), 29.to_string());
                let filter = KPFilter::new("fps", argument, vec![]).unwrap();
                graph.add_filter(vec![filter]).unwrap();
            }
            {
                let mut argument = BTreeMap::new();
                argument.insert("PTS-STARTPTS".to_string(), "".to_string());
                let filter = KPFilter::new("setpts", argument, vec![]).unwrap();
                graph.add_filter(vec![filter]).unwrap();
            }
        } else if media_type.eq(&KPAVMediaType::KPAVMEDIA_TYPE_AUDIO) {
            {
                let mut argument = BTreeMap::new();
                argument.insert("sample_fmts".to_string(), KPAVSampleFormat::from(AV_SAMPLE_FMT_FLTP).to_string());
                let filter = KPFilter::new("aformat", argument, vec![]).unwrap();
                graph.add_filter(vec![filter]).unwrap();
            }
            {
                let mut argument = BTreeMap::new();
                argument.insert("ocl".to_string(), 3.to_string());
                argument.insert("och".to_string(), 2.to_string());
                argument.insert("out_sample_rate".to_string(), 48000.to_string());
                let filter = KPFilter::new("aresample", argument, vec![]).unwrap();
                graph.add_filter(vec![filter]).unwrap();
            }
            {
                let mut argument = BTreeMap::new();
                argument.insert("r".to_string(), 48000.to_string());
                let filter = KPFilter::new("asetrate", argument, vec![]).unwrap();
                graph.add_filter(vec![filter]).unwrap();
            }
            {
                let mut argument = BTreeMap::new();
                argument.insert("PTS-STARTPTS".to_string(), "".to_string());
                let filter = KPFilter::new("asetpts", argument, vec![]).unwrap();
                graph.add_filter(vec![filter]).unwrap();
            }
        }

        if media_type == &scene.media_type {
            graph.add_scene(&scene)?;
        }

        graph.injection_sink().unwrap();
        graph_map.insert(media_type.clone(), graph);
    }

    let mut encode = KPEncode::new("flv", encode_parameter);
    encode.redirect_path("/tmp/main.flv");
    encode.open().unwrap();
    encode.write_header().unwrap();

    // set frame size
    if let Some(audio_graph) = graph_map.get_mut(&KPAVMediaType::KPAVMEDIA_TYPE_AUDIO) {
        audio_graph.set_frame_size(encode.get_audio_frame_size().unwrap()).unwrap();
    }

    for get_frame in decode.iter() {
        let (media_type, frame) = get_frame.unwrap();
        info!("decode frame. pts: {}, media_type: {}", frame.get().pts, media_type);

        // send to graph
        let graph = graph_map.get_mut(&media_type).unwrap();
        graph.stream_to_graph(frame).unwrap();
        for filter_frame in graph.iter() {
            let get_filter_frame = filter_frame.unwrap();
            info!("filter frame. pts: {}", get_filter_frame.get().pts);

            encode.stream_to_encode(get_filter_frame, &media_type).unwrap();

            // send to encode
            while let Some(packet) = encode.iter().next() {
                encode.write(&packet).unwrap()
            }
        }
    }

    for (media_type, graph) in graph_map.iter_mut() {
        graph.flush().unwrap();
        for filter_frame in graph.iter() {
            let get_filter_frame = filter_frame.unwrap();
            info!("filter frame. pts: {}", get_filter_frame.get().pts);

            encode.stream_to_encode(get_filter_frame, media_type).unwrap();

            // send to encode
            while let Some(packet) = encode.iter().next() {
                encode.write(&packet).unwrap()
            }
        }
    }

    encode.flush().unwrap();
    // send to encode
    while let Some(packet) = encode.iter().next() {
        encode.write(&packet).unwrap()
    }

    encode.write_trailer().unwrap();

    assert_eq!(decode.get_status(), &KPCodecStatus::Ended);
    for (_, graph) in graph_map.iter() {
        assert_eq!(graph.get_status(), &KPGraphStatus::Ended);
    }

    Ok(())
}

