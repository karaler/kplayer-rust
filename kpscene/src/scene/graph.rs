use std::time::Duration;
use log::debug;
use tokio::sync::broadcast::error::TryRecvError;
use crate::scene::*;

pub trait KPSceneGraph {
    fn add_scene(&mut self, scene: &KPScene, sort_type: KPSceneSortType) -> Result<()>;
    fn add_core(&mut self, media_type: &KPAVMediaType, encode_parameter: &BTreeMap<KPAVMediaType, KPEncodeParameter>) -> Result<()>;
}

impl KPSceneGraph for KPGraph {
    fn add_scene(&mut self, scene: &KPScene, sort_type: KPSceneSortType) -> Result<()> {
        for engine in scene.iter() {
            if self.get_media_type().eq(&engine.media_type) && engine.sort_type.eq(&sort_type) {
                for group in engine.groups.iter() {
                    self.add_filter(group.clone())?;
                }
            }
        }
        Ok(())
    }

    fn add_core(&mut self, media_type: &KPAVMediaType, encode_parameter: &BTreeMap<KPAVMediaType, KPEncodeParameter>) -> Result<()> {
        if media_type.eq(&KPAVMediaType::KPAVMEDIA_TYPE_VIDEO) {
            let default_param = KPEncodeParameter::default(&KPAVMediaType::KPAVMEDIA_TYPE_VIDEO);
            let (codec_id, width, height, pix_fmt, framerate, max_bitrate, quality, profile, preset, gop_uint, metadata) = encode_parameter.get(&KPAVMediaType::KPAVMEDIA_TYPE_VIDEO).unwrap_or(&default_param).get_video_parameter()?;
            {
                let mut argument = BTreeMap::new();
                argument.insert("w".to_string(), width.to_string());
                argument.insert("h".to_string(), height.to_string());
                let filter = KPFilter::new("scale", "scale", argument, vec![])?;
                self.add_filter(vec![filter])?;
            }
            {
                let mut argument = BTreeMap::new();
                argument.insert("pix_fmts".to_string(), pix_fmt.to_string());
                let filter = KPFilter::new("format", "format", argument, vec![])?;
                self.add_filter(vec![filter])?;
            }
            {
                let mut argument = BTreeMap::new();
                argument.insert("fps".to_string(), framerate.get_fps().to_string());
                let filter = KPFilter::new("fps", "fps", argument, vec![])?;
                self.add_filter(vec![filter])?;
            }
            {
                let mut argument = BTreeMap::new();
                argument.insert("PTS-STARTPTS".to_string(), "".to_string());
                let filter = KPFilter::new("setpts", "setpts", argument, vec![])?;
                self.add_filter(vec![filter])?;
            }
        } else if media_type.eq(&KPAVMediaType::KPAVMEDIA_TYPE_AUDIO) {
            let default_param = KPEncodeParameter::default(&KPAVMediaType::KPAVMEDIA_TYPE_AUDIO);
            let (codec_id, sample_rate, sample_fmt, channel_layout, channels, metadata) = encode_parameter.get(&KPAVMediaType::KPAVMEDIA_TYPE_AUDIO).unwrap_or(&default_param).get_audio_parameter()?;
            {
                let mut argument = BTreeMap::new();
                argument.insert("sample_fmts".to_string(), sample_fmt.to_string());
                let filter = KPFilter::new("aformat", "aformat", argument, vec![])?;
                self.add_filter(vec![filter])?;
            }
            {
                let mut argument = BTreeMap::new();
                argument.insert("ocl".to_string(), channel_layout.to_string());
                argument.insert("och".to_string(), channels.to_string());
                argument.insert("out_sample_rate".to_string(), sample_rate.to_string());
                let filter = KPFilter::new("aresample", "aresample", argument, vec![])?;
                self.add_filter(vec![filter])?;
            }
            {
                let mut argument = BTreeMap::new();
                argument.insert("r".to_string(), sample_rate.to_string());
                let filter = KPFilter::new("asetrate", "asetrate", argument, vec![])?;
                self.add_filter(vec![filter])?;
            }
            {
                let mut argument = BTreeMap::new();
                argument.insert("PTS-STARTPTS".to_string(), "".to_string());
                let filter = KPFilter::new("asetpts", "asetpts", argument, vec![])?;
                self.add_filter(vec![filter])?;
            }
        }
        Ok(())
    }
}

#[tokio::test]
async fn load_plugin() -> Result<()> {
    initialize();

    let mut decode = KPDecode::new(env::var("INPUT_PATH")?);
    decode.open()?;

    // set expect stream
    let mut expect_streams = HashMap::new();
    expect_streams.insert(KPAVMediaType::KPAVMEDIA_TYPE_VIDEO, None);
    expect_streams.insert(KPAVMediaType::KPAVMEDIA_TYPE_AUDIO, None);
    decode.set_expect_stream(expect_streams.clone());
    decode.find_streams()?;
    decode.open_codec()?;

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
    let wasm_path = env::var("TEXT_WASM_PATH")?;
    let file_data = fs::read(wasm_path)?;
    let mut scene = KPScene::new();
    scene.add_engine("app", KPEngine::new(file_data, Default::default()).await?);

    // create graph
    let mut graph_map = HashMap::new();
    for (media_type, _) in expect_streams.iter() {
        let mut graph = KPGraph::new(media_type);
        graph.injection_source(&decode)?;

        // add before scene
        graph.add_scene(&scene, KPSceneSortType::Before)?;

        // add core
        graph.add_core(media_type, &encode_parameter)?;

        // add before scene
        graph.add_scene(&scene, KPSceneSortType::After)?;

        graph.injection_sink()?;
        graph_map.insert(media_type.clone(), graph);
    }

    let mut encode = KPEncode::new("flv", encode_parameter);
    encode.redirect_path("/tmp/main.flv");
    encode.open()?;
    encode.write_header()?;

    // set frame size
    if let Some(audio_graph) = graph_map.get_mut(&KPAVMediaType::KPAVMEDIA_TYPE_AUDIO) {
        audio_graph.set_frame_size(encode.get_audio_frame_size()?)?;
    }

    for get_frame in decode.iter() {
        let (media_type, frame) = get_frame?;
        info!("decode frame. pts: {}, media_type: {}", frame.get().pts, media_type);

        // send to graph
        let graph = graph_map.get_mut(&media_type).unwrap();
        graph.stream_to_graph(frame)?;
        for filter_frame in graph.iter() {
            let get_filter_frame = filter_frame?;
            info!("filter frame. pts: {}", get_filter_frame.get().pts);

            encode.stream_to_encode(get_filter_frame, &media_type)?;

            // send to encode
            while let Some(packet) = encode.iter().next() {
                encode.write(&packet)?
            }
        }
    }

    for (media_type, graph) in graph_map.iter_mut() {
        graph.flush()?;
        for filter_frame in graph.iter() {
            let get_filter_frame = filter_frame?;
            info!("filter frame. pts: {}", get_filter_frame.get().pts);

            encode.stream_to_encode(get_filter_frame, media_type)?;

            // send to encode
            while let Some(packet) = encode.iter().next() {
                encode.write(&packet)?
            }
        }
    }

    encode.flush()?;
    // send to encode
    while let Some(packet) = encode.iter().next() {
        encode.write(&packet)?
    }

    encode.write_trailer()?;

    assert_eq!(decode.get_status(), &KPCodecStatus::Ended);
    for (_, graph) in graph_map.iter() {
        assert_eq!(graph.get_status(), &KPGraphStatus::Ended);
    }

    Ok(())
}


#[tokio::test]
async fn update_plugin_argument() -> Result<()> {
    initialize();

    let mut decode = KPDecode::new(env::var("INPUT_PATH")?);
    decode.open()?;

    // set expect stream
    let mut expect_streams = HashMap::new();
    expect_streams.insert(KPAVMediaType::KPAVMEDIA_TYPE_VIDEO, None);
    expect_streams.insert(KPAVMediaType::KPAVMEDIA_TYPE_AUDIO, None);
    decode.set_expect_stream(expect_streams.clone());
    decode.find_streams()?;
    decode.open_codec()?;

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
    let wasm_path = env::var("TEXT_WASM_PATH")?;
    let file_data = fs::read(wasm_path).expect("plugin file not found");
    let mut scene = KPScene::new();
    scene.add_engine("app", KPEngine::new(file_data, Default::default()).await?);

    // create graph
    let mut graph_map = HashMap::new();
    for (media_type, _) in expect_streams.iter() {
        let mut graph = KPGraph::new(media_type);
        graph.injection_source(&decode)?;

        // add before scene
        graph.add_scene(&scene, KPSceneSortType::Before)?;

        // add core
        graph.add_core(media_type, &encode_parameter)?;

        // add before scene
        graph.add_scene(&scene, KPSceneSortType::After)?;

        graph.injection_sink()?;
        graph_map.insert(media_type.clone(), graph);
    }

    let mut encode = KPEncode::new("flv", encode_parameter);
    encode.redirect_path("/tmp/main.flv");
    encode.open()?;
    encode.write_header()?;

    // set frame size
    if let Some(audio_graph) = graph_map.get_mut(&KPAVMediaType::KPAVMEDIA_TYPE_AUDIO) {
        audio_graph.set_frame_size(encode.get_audio_frame_size()?)?;
    }

    // async update argument
    let (tx, rx) = tokio::sync::broadcast::channel::<i32>(100);

    // sleep send msg
    let tx_clone = tx.clone();
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(5)).await;
        tx_clone.send(1).unwrap();
        info!("send message");
    });

    let transcode = tokio::task::spawn_blocking(move || {
        let mut receiver = tx.subscribe();
        for get_frame in decode.iter() {
            // Receive messages from msg_receiver
            loop {
                let msg = match receiver.try_recv() {
                    Err(_) => break,
                    Ok(msg) => {
                        info!("receive message {}",msg);
                        let graph = graph_map.get_mut(&KPAVMediaType::KPAVMEDIA_TYPE_VIDEO).unwrap();
                        let mut update_arguments = BTreeMap::new();
                        update_arguments.insert("text".to_string(), "changed".to_string());
                        let cmd = scene.get_update_argument("app", update_arguments)?;
                        graph.send_command(cmd)?;
                    }
                };
            }

            // process frame
            let (media_type, frame) = get_frame?;
            debug!("decode frame. pts: {}, media_type: {}", frame.get().pts, media_type);

            // send to graph
            let graph = graph_map.get_mut(&media_type).unwrap();
            graph.stream_to_graph(frame)?;
            for filter_frame in graph.iter() {
                let get_filter_frame = filter_frame?;
                debug!("filter frame. pts: {}", get_filter_frame.get().pts);

                encode.stream_to_encode(get_filter_frame, &media_type)?;

                // send to encode
                while let Some(packet) = encode.iter().next() {
                    encode.write(&packet)?
                }
            }
        }

        for (media_type, graph) in graph_map.iter_mut() {
            graph.flush()?;
            for filter_frame in graph.iter() {
                let get_filter_frame = filter_frame?;
                debug!("filter frame. pts: {}", get_filter_frame.get().pts);

                encode.stream_to_encode(get_filter_frame, media_type)?;

                // send to encode
                while let Some(packet) = encode.iter().next() {
                    encode.write(&packet)?
                }
            }
        }

        encode.flush()?;
        // send to encode
        while let Some(packet) = encode.iter().next() {
            encode.write(&packet)?
        }

        encode.write_trailer()?;

        assert_eq!(decode.get_status(), &KPCodecStatus::Ended);
        for (_, graph) in graph_map.iter() {
            assert_eq!(graph.get_status(), &KPGraphStatus::Ended);
        }

        Ok(())
    });
    transcode.await?
}