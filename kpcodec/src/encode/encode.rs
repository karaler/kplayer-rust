use crate::decode::decode::KPDecodeIterator;
use crate::encode::*;

const WARN_QUEUE_LIMIT: usize = 500;

#[derive(Default, Debug)]
pub struct KPEncodeStreamContext {
    media_type: KPAVMediaType,
    time_base: KPAVRational,
    codec_context_ptr: KPAVCodecContext,
    end_of_file: bool,
    metadata: BTreeMap<String, String>,
    packets: VecDeque<KPAVPacket>,
}

pub struct KPEncodeIterator<'a> {
    encode: &'a mut KPEncode,
}

impl<'a> Iterator for KPEncodeIterator<'a> {
    type Item = KPAVPacket;

    fn next(&mut self) -> Option<Self::Item> {
        let encode = &mut self.encode;

        // stream to queue
        encode.stream_from_encode().unwrap();

        let lead_stream_index = encode.lead_stream_index;
        if let Some((_, lead_dts)) = encode.maintainer {
            if encode.streams.values().any(|v| !v.end_of_file && v.packets.len() == 0) { return None; }
            // get packet from queue
            for (follow_stream_index, follow_stream_context) in encode.streams.iter_mut() {
                if lead_stream_index.eq(follow_stream_index) { continue; }
                let follow_packet = follow_stream_context.packets.pop_front();
                if let Some(pkt) = follow_packet {
                    if pkt.get().dts > lead_dts {
                        trace!("renew packet to queue. stream_index: {}, packet: {}", follow_stream_index, pkt);
                        follow_stream_context.packets.push_front(pkt);
                        continue;
                    } else {
                        trace!("iterator packet. packet: {}", pkt);
                        return Some(pkt);
                    }
                } else {
                    if !follow_stream_context.end_of_file { return None; }
                }
            };
        };

        // send lead
        match encode.streams.get_mut(&lead_stream_index).and_then(|ctx| ctx.packets.pop_front()) {
            None => None,
            Some(pkt) => {
                encode.maintainer = Some((pkt.get().pts, pkt.get().dts));
                trace!("iterator packet. packet: {}", pkt);
                Some(pkt)
            }
        }
    }
}

#[derive(Default)]
pub struct KPEncode {
    output_format: String,
    output_path: String,

    // formation
    format_context_options: HashMap<String, String>,
    format_context_ptr: KPAVFormatContext,

    // options
    encode_parameter: BTreeMap<KPAVMediaType, KPEncodeParameter>,
    streams: BTreeMap<usize, KPEncodeStreamContext>,
    metadata: BTreeMap<String, String>,

    // state
    lead_stream_index: usize,
    status: KPCodecStatus,
    maintainer: Option<(i64, i64)>,
    position: Duration,
}

impl KPEncode {
    pub fn new<T: ToString>(output_format: T, encode_parameter: BTreeMap<KPAVMediaType, KPEncodeParameter>) -> Self {
        let mut format_context_options = HashMap::new();
        format_context_options.insert("rw_timeout".to_string(), "10".to_string());
        KPEncode {
            output_format: output_format.to_string(),
            status: KPCodecStatus::None,
            encode_parameter,
            format_context_options,
            output_path: String::from("/dev/null"),
            ..Default::default()
        }
    }

    pub fn redirect_path<T: ToString>(&mut self, output_path: T) {
        assert_eq!(self.status, KPCodecStatus::None);
        self.output_path = output_path.to_string();
        debug!("redirect output path. path: {}", self.output_path);
    }

    pub fn open(&mut self) -> Result<()> {
        assert_eq!(self.status, KPCodecStatus::None);

        let output_format = unsafe { av_guess_format(cstring!(self.output_format).as_ptr(), ptr::null_mut(), ptr::null_mut()) };
        if output_format.is_null() { return Err(anyhow!("guess output format failed. error: {}", self.output_format)); };

        assert!(self.format_context_ptr.is_null());
        let mut format_context_ptr: *mut AVFormatContext = ptr::null_mut();
        let ret = unsafe { avformat_alloc_output_context2(&mut format_context_ptr, output_format, ptr::null_mut(), ptr::null_mut()) };
        if ret < 0 { return Err(anyhow!("alloc output formation failed. error: {:?}", averror!(ret))); }
        self.format_context_ptr.set(format_context_ptr);
        assert!(!self.format_context_ptr.is_null());

        // open file
        let mut open_options = KPAVDictionary::new(&self.format_context_options);
        let mut open_options_ptr = open_options.get();
        let ret = unsafe { avio_open2(&mut self.format_context_ptr.get().pb, cstring!(self.output_path).as_ptr(), AVIO_FLAG_WRITE as c_int, ptr::null_mut(), &mut open_options_ptr) };
        if ret < 0 { return Err(anyhow!("open output file failed. error: {:?}", averror!(ret))); }
        open_options.set(open_options_ptr);
        assert_eq!(open_options.get(), open_options_ptr);
        info!("open output path success. path: {}, format: {}", self.output_path, self.output_format);

        // set metadata
        for (k, v) in self.metadata.iter() {
            unsafe { av_dict_set(&mut self.format_context_ptr.get().metadata, cstring!(k).as_ptr(), cstring!(v).as_ptr(), 0) };
        }

        // create codec context closure
        let create_codec_context = |codec_id: &KPAVCodecId| -> Result<KPAVCodecContext>{
            assert!(!codec_id.is_none());
            let ret = unsafe { avformat_query_codec(output_format, codec_id.get(), 0) };
            if ret < 0 { return Err(anyhow!("the output format not support encoder. error:{:?}", averror!(ret))); }

            let codec = unsafe { avcodec_find_encoder(codec_id.get()) };
            if codec.is_null() {
                return Err(anyhow!("find encoder failed. encoder: {}", codec_id));
            }
            let codec_context = KPAVCodecContext::new(codec);

            assert!(!self.format_context_ptr.get().oformat.is_null());
            if unsafe { *self.format_context_ptr.get().oformat }.flags & AVFMT_GLOBALHEADER as c_int != 0 {
                codec_context.get().flags |= AV_CODEC_FLAG_GLOBAL_HEADER as c_int;
            }
            assert_ne!(codec_context.get().flags & AV_CODEC_FLAG_GLOBAL_HEADER as c_int, 0);
            Ok(codec_context)
        };

        // open codec
        for (media_type, param) in self.encode_parameter.iter() {
            let (codec_context, media_type, metadata) = match param {
                KPEncodeParameter::Video { codec_id, width, height, pix_fmt, framerate, max_bitrate, quality, profile, preset, gop_uint, metadata } => {
                    let codec_context = create_codec_context(codec_id)?;
                    let codec = codec_context.get().codec;

                    // set video encode params
                    codec_context.get().width = width.clone() as i32;
                    codec_context.get().height = height.clone() as i32;
                    codec_context.get().pix_fmt = pix_fmt.get();
                    codec_context.get().framerate = framerate.get();
                    codec_context.get().time_base = av_inv_q(framerate.get());
                    codec_context.get().gop_size = gop_uint.clone() as c_int * framerate.get().num;

                    assert!(!codec.is_null());
                    let codec_name = cstr!(unsafe{(*codec).name});
                    match codec_name {
                        c if c.eq("libx264") => {
                            unsafe {
                                let codec_context_ref = codec_context.get();
                                av_opt_set(codec_context_ref.priv_data, cstring!("profile").as_ptr(), cstring!(profile.to_string()).as_ptr(), AV_OPT_SEARCH_CHILDREN as c_int);
                                av_opt_set(codec_context_ref.priv_data, cstring!("preset").as_ptr(), cstring!(preset.to_string()).as_ptr(), AV_OPT_SEARCH_CHILDREN as c_int);
                                av_opt_set(codec_context_ref.priv_data, cstring!("x264opts").as_ptr(), cstring!("force-cfr").as_ptr(), AV_OPT_SEARCH_CHILDREN as c_int);
                                if quality.clone() != 0 {
                                    av_opt_set(codec_context_ref.priv_data, cstring!("crf").as_ptr(), cstring!(quality.to_string()).as_ptr(), AV_OPT_SEARCH_CHILDREN as c_int);
                                }
                                if max_bitrate.clone() > 0 {
                                    av_opt_set(codec_context_ref.priv_data, cstring!("maxrate").as_ptr(), cstring!(max_bitrate.to_string()).as_ptr(), AV_OPT_SEARCH_CHILDREN as c_int);
                                }
                            }
                        }
                        c if c.eq("openh264") => {
                            unsafe {
                                let codec_context_ref = codec_context.get();
                                av_opt_set(codec_context_ref.priv_data, cstring!("profile").as_ptr(), cstring!(profile.to_string()).as_ptr(), AV_OPT_SEARCH_CHILDREN as c_int);
                                av_opt_set(codec_context_ref.priv_data, cstring!("preset").as_ptr(), cstring!(preset.to_string()).as_ptr(), AV_OPT_SEARCH_CHILDREN as c_int);
                                if quality.clone() != 0 {
                                    av_opt_set(codec_context_ref.priv_data, cstring!("qp").as_ptr(), cstring!(quality.to_string()).as_ptr(), AV_OPT_SEARCH_CHILDREN as c_int);
                                }
                                if max_bitrate.clone() > 0 {
                                    av_opt_set(codec_context_ref.priv_data, cstring!("max_bitrate").as_ptr(), cstring!(max_bitrate.to_string()).as_ptr(), AV_OPT_SEARCH_CHILDREN as c_int);
                                }
                            }
                        }
                        _ => {}
                    }
                    (codec_context, KPAVMediaType::KPAVMEDIA_TYPE_VIDEO, metadata)
                }
                KPEncodeParameter::Audio { codec_id, sample_rate, sample_fmt, channel_layout, channels, metadata } => {
                    let codec_context = create_codec_context(codec_id)?;
                    codec_context.get().sample_rate = sample_rate.clone() as c_int;
                    codec_context.get().channel_layout = channel_layout.clone() as u64;
                    codec_context.get().channels = channels.clone() as c_int;
                    codec_context.get().sample_fmt = sample_fmt.get();

                    (codec_context, KPAVMediaType::KPAVMEDIA_TYPE_AUDIO, metadata)
                }
            };

            // add stream
            let stream = unsafe { avformat_new_stream(self.format_context_ptr.get(), codec_context.get().codec) };
            if stream.is_null() { return Err(anyhow!("add new stream failed. param: {:?}", param)); }
            let stream_ref = unsafe { stream.as_mut().unwrap() };
            if media_type == KPAVMediaType::KPAVMEDIA_TYPE_VIDEO {
                self.lead_stream_index = stream_ref.index as usize;
            }
            debug!("create stream success. media_type: {}", media_type);

            // set metadata
            for (k, v) in metadata.iter() {
                unsafe { av_dict_set(&mut stream_ref.metadata, cstring!(k).as_ptr(), cstring!(v).as_ptr(), 0) };
            }

            let ret = unsafe { avcodec_open2(codec_context.get(), codec_context.get().codec, ptr::null_mut()) };
            if ret < 0 { return Err(anyhow!("open codec failed. media_type: {}, error: {:?}", media_type, averror!(ret))); }

            let ret = unsafe { avcodec_parameters_from_context(stream_ref.codecpar, codec_context.get()) };
            if ret < 0 { return Err(anyhow!("copy parameter to stream failed. media_type: {}, error: {:?}", media_type, averror!(ret))); }
            assert!(!stream_ref.codec.is_null());
            debug!("open encoder. codec: {}, media_type: {}",KPAVCodecId::from(unsafe{*stream_ref.codec}.codec_id) , media_type);

            self.streams.insert(stream_ref.index as usize, KPEncodeStreamContext {
                media_type,
                time_base: KPAVRational::from(stream_ref.time_base),
                codec_context_ptr: codec_context,
                end_of_file: false,
                metadata: metadata.clone(),
                packets: Default::default(),
            });
        }

        self.status = KPCodecStatus::Opened;
        Ok(())
    }

    pub fn write_header(&mut self) -> Result<()> {
        assert_eq!(self.status, KPCodecStatus::Opened);
        assert!(self.streams.iter().all(|(_, stream)| !stream.end_of_file));

        let ret = unsafe { avformat_write_header(self.format_context_ptr.get(), ptr::null_mut()) };
        if ret < 0 { return Err(anyhow!("write header failed. error: {:?}", averror!(ret))); }

        // avformat_write_header will update stream context
        // update stream context when write header
        let format_context = self.format_context_ptr.get();
        for (stream_index, stream_context) in self.streams.iter_mut() {
            let stream = unsafe { (*(format_context.streams.add(stream_index.clone()))).as_mut().unwrap() };
            stream_context.time_base = KPAVRational::from(stream.time_base);
        }

        self.status = KPCodecStatus::Started;
        Ok(())
    }

    pub fn write_trailer(&mut self) -> Result<()> {
        assert_eq!(self.status, KPCodecStatus::Stopped);
        assert!(self.streams.iter().all(|(_, stream)| stream.end_of_file));

        let ret = unsafe { av_write_trailer(self.format_context_ptr.get()) };
        if ret < 0 { return Err(anyhow!("write trailer failed. error: {:?}", averror!(ret))); }

        let ret = unsafe { avformat_flush(self.format_context_ptr.get()) };
        if ret < 0 { return Err(anyhow!("flush failed. error: {:?}", averror!(ret))); }

        self.status = KPCodecStatus::Ended;
        Ok(())
    }

    pub fn flush(&mut self) -> Result<()> {
        for (stream_index, stream_context) in self.streams.iter_mut() {
            let ret = unsafe { avcodec_send_frame(stream_context.codec_context_ptr.get(), ptr::null_mut()) };
            if ret < 0 { return Err(anyhow!("stream to encode failed. error: {:?}", averror!(ret))); }
            if ret == AVERROR_EOF {
                stream_context.codec_context_ptr.flush()?;
                debug!("stream index flushed. stream index: {}", stream_index);
                return Ok(());
            }
        }
        Ok(())
    }

    pub fn stream_to_encode(&mut self, frame: KPAVFrame, media_type: &KPAVMediaType) -> Result<()> {
        assert!(!frame.is_empty());
        assert_eq!(self.status, KPCodecStatus::Started);
        assert!(self.streams.iter().any(|(_, ctx)| { &ctx.media_type == media_type }));

        let (stream_index, stream_context) = self.streams.iter_mut().find(|(_, stream)| { &stream.media_type == media_type }).unwrap();
        assert!(!stream_context.end_of_file);

        let ret = unsafe { avcodec_send_frame(stream_context.codec_context_ptr.get(), frame.get()) };
        if ret < 0 { return Err(anyhow!("stream to encode failed. error: {:?}", averror!(ret))); }

        Ok(())
    }

    pub fn stream_from_encode(&mut self) -> Result<()> {
        assert!(matches!(self.status, KPCodecStatus::Started | KPCodecStatus::Stopped));
        if !self.streams.values().any(|v| {
            v.end_of_file == false
        }) {
            self.status = KPCodecStatus::Stopped;
            return Ok(());
        }

        for (stream_index, stream_context) in self.streams.iter_mut() {
            if stream_context.end_of_file { continue; };

            // get packet
            loop {
                let packet = KPAVPacket::new();
                let ret = unsafe { avcodec_receive_packet(stream_context.codec_context_ptr.get(), packet.get()) };
                match ret {
                    r if r >= 0 => {
                        if stream_context.packets.len() >= WARN_QUEUE_LIMIT {
                            warn!("codec context queue length overlong. size: {}, media_type:{}", stream_context.packets.len(), stream_context.media_type);
                        }
                        trace!("receipt packet. index:{}, media_type:{}, pts:{}, dts: {}", stream_index, stream_context.media_type, packet.get().pts, packet.get().dts);

                        // transform packet
                        unsafe { av_packet_rescale_ts(packet.get(), stream_context.codec_context_ptr.get().time_base, stream_context.time_base.get()) };
                        packet.get().stream_index = stream_index.clone() as c_int;
                        trace!("transform packet. index:{}, media_type:{}, pts:{}, dts: {}", stream_index, stream_context.media_type, packet.get().pts, packet.get().dts);


                        // push packet
                        assert!(packet.is_valid());
                        stream_context.packets.push_back(packet);
                    }
                    r if r == AVERROR(EAGAIN) => {
                        break;
                    }
                    r if r == AVERROR_EOF => {
                        debug!("stream from encode end of file. stream index: {}", stream_index);
                        stream_context.end_of_file = true;
                        break;
                    }
                    _ => {
                        return Err(anyhow!("receive packet failed. error: {:?}", averror!(ret)));
                    }
                };
            }
        }

        Ok(())
    }

    pub fn write(&mut self, packet: &KPAVPacket) -> Result<()> {
        assert!(matches!(self.status, KPCodecStatus::Started | KPCodecStatus::Stopped));
        assert!(packet.is_valid());

        debug!("write to output file packet. packet: {}", packet);
        let ret = unsafe { av_interleaved_write_frame(self.format_context_ptr.get(), packet.get()) };
        if ret < 0 {
            return Err(anyhow!("write packet failed. error: {:?}", averror!(ret)));
        }
        Ok(())
    }

    pub fn get_audio_frame_size(&self) -> Result<usize> {
        assert!(matches!(self.status, KPCodecStatus::Opened | KPCodecStatus::Started));
        assert!(self.encode_parameter.get(&KPAVMediaType::KPAVMEDIA_TYPE_AUDIO).is_some());

        match self.streams.iter().find(|(index, item)| {
            item.media_type == KPAVMediaType::KPAVMEDIA_TYPE_AUDIO
        }) {
            None => Err(anyhow!("not support audio stream index")),
            Some((_, audio_stream_context)) => {
                Ok(audio_stream_context.codec_context_ptr.get().frame_size as usize)
            }
        }
    }
}

impl KPEncode {
    pub fn iter(&mut self) -> KPEncodeIterator {
        KPEncodeIterator {
            encode: self,
        }
    }
}

#[test]
fn test_encode() {
    use crate::decode::decode::KPDecode;

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
}