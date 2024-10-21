use crate::scene::*;

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
    let scene = KPScene::new(KPEngine::new(file_data).await?);

    // create graph
    let mut graph_map = HashMap::new();
    for (media_type, _) in expect_streams.iter() {
        let mut graph = KPGraph::new(media_type);
        graph.injection_source(&decode)?;

        if scene.media_type.eq(media_type) && scene.sort_type.eq(&KPSceneSortType::Before) {
            graph.add_scene(&scene)?;
        }

        if media_type.eq(&KPAVMediaType::KPAVMEDIA_TYPE_VIDEO) {
            {
                let mut argument = BTreeMap::new();
                argument.insert("w".to_string(), "848".to_string());
                argument.insert("h".to_string(), "480".to_string());
                let filter = KPFilter::new("scale", "scale", argument, vec![])?;
                graph.add_filter(vec![filter])?;
            }
            {
                let mut argument = BTreeMap::new();
                argument.insert("pix_fmts".to_string(), KPAVPixelFormat::from(AV_PIX_FMT_YUV420P).to_string());
                let filter = KPFilter::new("format", "format", argument, vec![])?;
                graph.add_filter(vec![filter])?;
            }
            {
                let mut argument = BTreeMap::new();
                argument.insert("fps".to_string(), 29.to_string());
                let filter = KPFilter::new("fps", "fps", argument, vec![])?;
                graph.add_filter(vec![filter])?;
            }
            {
                let mut argument = BTreeMap::new();
                argument.insert("PTS-STARTPTS".to_string(), "".to_string());
                let filter = KPFilter::new("setpts", "setpts", argument, vec![])?;
                graph.add_filter(vec![filter])?;
            }
        } else if media_type.eq(&KPAVMediaType::KPAVMEDIA_TYPE_AUDIO) {
            {
                let mut argument = BTreeMap::new();
                argument.insert("sample_fmts".to_string(), KPAVSampleFormat::from(AV_SAMPLE_FMT_FLTP).to_string());
                let filter = KPFilter::new("aformat", "aformat", argument, vec![])?;
                graph.add_filter(vec![filter])?;
            }
            {
                let mut argument = BTreeMap::new();
                argument.insert("ocl".to_string(), 3.to_string());
                argument.insert("och".to_string(), 2.to_string());
                argument.insert("out_sample_rate".to_string(), 48000.to_string());
                let filter = KPFilter::new("aresample", "aresample", argument, vec![])?;
                graph.add_filter(vec![filter])?;
            }
            {
                let mut argument = BTreeMap::new();
                argument.insert("r".to_string(), 48000.to_string());
                let filter = KPFilter::new("asetrate", "asetrate", argument, vec![])?;
                graph.add_filter(vec![filter])?;
            }
            {
                let mut argument = BTreeMap::new();
                argument.insert("PTS-STARTPTS".to_string(), "".to_string());
                let filter = KPFilter::new("asetpts", "asetpts", argument, vec![])?;
                graph.add_filter(vec![filter])?;
            }
        }

        if scene.media_type.eq(media_type) && scene.sort_type.eq(&KPSceneSortType::After) {
            graph.add_scene(&scene)?;
        }

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
    let scene = KPScene::new(KPEngine::new(file_data).await?);

    // create graph
    let mut graph_map = HashMap::new();
    for (media_type, _) in expect_streams.iter() {
        let mut graph = KPGraph::new(media_type);
        graph.injection_source(&decode)?;

        if scene.media_type.eq(media_type) && scene.sort_type.eq(&KPSceneSortType::Before) {
            graph.add_scene(&scene)?;
        }

        if media_type.eq(&KPAVMediaType::KPAVMEDIA_TYPE_VIDEO) {
            {
                let mut argument = BTreeMap::new();
                argument.insert("w".to_string(), "848".to_string());
                argument.insert("h".to_string(), "480".to_string());
                let filter = KPFilter::new("scale", "scale", argument, vec![])?;
                graph.add_filter(vec![filter])?;
            }
            {
                let mut argument = BTreeMap::new();
                argument.insert("pix_fmts".to_string(), KPAVPixelFormat::from(AV_PIX_FMT_YUV420P).to_string());
                let filter = KPFilter::new("format", "format", argument, vec![])?;
                graph.add_filter(vec![filter])?;
            }
            {
                let mut argument = BTreeMap::new();
                argument.insert("fps".to_string(), 29.to_string());
                let filter = KPFilter::new("fps", "fps", argument, vec![])?;
                graph.add_filter(vec![filter])?;
            }
            {
                let mut argument = BTreeMap::new();
                argument.insert("PTS-STARTPTS".to_string(), "".to_string());
                let filter = KPFilter::new("setpts", "setpts", argument, vec![])?;
                graph.add_filter(vec![filter])?;
            }
        } else if media_type.eq(&KPAVMediaType::KPAVMEDIA_TYPE_AUDIO) {
            {
                let mut argument = BTreeMap::new();
                argument.insert("sample_fmts".to_string(), KPAVSampleFormat::from(AV_SAMPLE_FMT_FLTP).to_string());
                let filter = KPFilter::new("aformat", "aformat", argument, vec![])?;
                graph.add_filter(vec![filter])?;
            }
            {
                let mut argument = BTreeMap::new();
                argument.insert("ocl".to_string(), 3.to_string());
                argument.insert("och".to_string(), 2.to_string());
                argument.insert("out_sample_rate".to_string(), 48000.to_string());
                let filter = KPFilter::new("aresample", "aresample", argument, vec![])?;
                graph.add_filter(vec![filter])?;
            }
            {
                let mut argument = BTreeMap::new();
                argument.insert("r".to_string(), 48000.to_string());
                let filter = KPFilter::new("asetrate", "asetrate", argument, vec![])?;
                graph.add_filter(vec![filter])?;
            }
            {
                let mut argument = BTreeMap::new();
                argument.insert("PTS-STARTPTS".to_string(), "".to_string());
                let filter = KPFilter::new("asetpts", "asetpts", argument, vec![])?;
                graph.add_filter(vec![filter])?;
            }
        }

        if scene.media_type.eq(media_type) && scene.sort_type.eq(&KPSceneSortType::After) {
            graph.add_scene(&scene)?;
        }

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

    let mut count = 0;
    for get_frame in decode.iter() {
        count = count + 1;
        if count == 5000 {
            // Receive messages from msg_receiver
            let graph = graph_map.get_mut(&KPAVMediaType::KPAVMEDIA_TYPE_VIDEO).unwrap();
            let mut update_arguments = BTreeMap::new();
            update_arguments.insert("text".to_string(), "changed".to_string());
            let cmd = scene.get_update_argument(update_arguments).await?;
            graph.send_command(cmd)?;
        }

        // process frame
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