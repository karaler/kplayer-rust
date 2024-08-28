use crate::filter::*;

const WARN_QUEUE_LIMIT: usize = 500;

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

#[derive(Default, Eq, PartialEq, Debug)]
pub enum KPGraphStatus {
    #[default]
    None,
    Created,
    Initialized,
    Opened,
    Ended,
}

#[derive(Default)]
pub struct KPGraphChain {
    filter: KPFilter,
    filter_context: KPAVFilterContext,
}

#[derive(Default)]
pub struct KPGraph {
    filter_graph: KPAVFilterGraph,
    filter_chain: Vec<Vec<KPGraphChain>>,
    status: KPGraphStatus,
    media_type: KPAVMediaType,
    frames: VecDeque<KPAVFrame>,
    audio_frame_size: Option<usize>,
}

pub struct KPGraphIterator<'a> {
    graph: &'a mut KPGraph,
}

impl<'a> Iterator for KPGraphIterator<'a> {
    type Item = (Result<KPAVFrame>);

    fn next(&mut self) -> Option<Self::Item> {
        let graph = &mut self.graph;
        if let Some(frame) = graph.frames.pop_back() {
            return Some(Ok(frame));
        }

        // stream to queue
        if let Err(err) = graph.stream_from_graph() {
            return Some(Err(err));
        }
        None
    }
}

impl KPGraph {
    pub fn iter(&mut self) -> KPGraphIterator {
        KPGraphIterator {
            graph: self,
        }
    }
}

impl KPGraph {
    pub fn new(media_type: &KPAVMediaType) -> Self {
        KPGraph {
            filter_graph: KPAVFilterGraph::new(),
            media_type: media_type.clone(),
            status: Default::default(),
            filter_chain: Default::default(),
            frames: Default::default(),
            audio_frame_size: None,
        }
    }

    pub fn injection_source(&mut self, source: &dyn KPGraphSource) -> Result<()> {
        assert_eq!(self.status, KPGraphStatus::None);
        assert!(!self.media_type.is_unknown());
        match source.get_source(&self.media_type)? {
            KPGraphSourceAttribute::Video { width, height, pix_fmt, time_base, frame_rate, pixel_aspect } => {
                assert_eq!(self.media_type, KPAVMediaType::from(AVMEDIA_TYPE_VIDEO));
                let mut arguments = HashMap::new();
                arguments.insert("width".to_string(), width.to_string());
                arguments.insert("height".to_string(), height.to_string());
                arguments.insert("pix_fmt".to_string(), pix_fmt.as_str()); // using as_str for source value
                arguments.insert("time_base".to_string(), time_base.to_string());
                arguments.insert("frame_rate".to_string(), frame_rate.to_string());
                arguments.insert("pixel_aspect".to_string(), pixel_aspect.to_string());
                let filter = KPFilter::new("buffer", arguments)?;
                self.add_filter(vec![filter])?;
            }
            KPGraphSourceAttribute::Audio { sample_rate, sample_fmt, channel_layout, channels, time_base } => {
                assert_eq!(self.media_type, KPAVMediaType::from(AVMEDIA_TYPE_AUDIO));
                let mut arguments = HashMap::new();
                arguments.insert("sample_rate".to_string(), sample_rate.to_string());
                arguments.insert("sample_fmt".to_string(), sample_fmt.as_str()); // using as_str for source value
                arguments.insert("channel_layout".to_string(), channel_layout.to_string());
                arguments.insert("channels".to_string(), channels.to_string());
                arguments.insert("time_base".to_string(), time_base.to_string());
                let filter = KPFilter::new("abuffer", arguments)?;
                self.add_filter(vec![filter])?;
            }
        }

        self.status = KPGraphStatus::Created;
        Ok(())
    }

    pub fn injection_sink(&mut self) -> Result<()> {
        assert_eq!(self.status, KPGraphStatus::Created);
        assert!(!self.media_type.is_unknown());
        match self.media_type.clone() {
            r if r == KPAVMediaType::from(AVMEDIA_TYPE_VIDEO) => {
                let filter = KPFilter::new("buffersink", HashMap::new())?;
                self.add_filter(vec![filter])?;
            }
            r if r == KPAVMediaType::from(AVMEDIA_TYPE_AUDIO) => {
                let filter = KPFilter::new("abuffersink", HashMap::new())?;
                self.add_filter(vec![filter])?;
            }
            _ => {}
        }

        self.status = KPGraphStatus::Initialized;

        // link graph
        self.link()?;

        Ok(())
    }

    pub fn add_filter(&mut self, filter: Vec<KPFilter>) -> Result<()> {
        // create filter_context
        let mut filter_chains = Vec::new();
        for f in filter.iter() {
            let filter_context = f.create_by_graph(&self.filter_graph)?;
            assert!(!filter_context.is_null());
            filter_chains.push(KPGraphChain { filter: f.clone(), filter_context })
        }

        // validate
        match self.status {
            KPGraphStatus::None => {
                assert_eq!(self.filter_chain.len(), 0);
            }
            KPGraphStatus::Created => {
                assert_eq!(self.filter_chain.len(), 1);
            }
            KPGraphStatus::Initialized => {
                assert!(self.filter_chain.len() >= 2);
                let last_filter = self.filter_chain.last().unwrap();
                let output_pads: usize = last_filter.iter().map(|inner| inner.filter_context.get_output_count()).sum();
                let input_pads: usize = filter_chains.iter().map(|inner| inner.filter_context.get_input_count()).sum();
                if output_pads != input_pads {
                    return Err(anyhow!("mismatch input and output pads. outputs:{}, inputs:{}", output_pads, input_pads));
                }
            }
            _ => {}
        }

        // append filter
        self.filter_chain.push(filter_chains);

        Ok(())
    }

    // only on audio graph
    pub fn set_frame_size(&mut self, frame_size: usize) -> Result<()> {
        assert_eq!(self.status, KPGraphStatus::Initialized);
        let sink = self.filter_chain.last().unwrap().first().unwrap();
        unsafe { av_buffersink_set_frame_size(sink.filter_context.get(), frame_size as c_uint) };
        self.audio_frame_size = Some(frame_size);
        Ok(())
    }

    fn link(&mut self) -> Result<()> {
        assert_eq!(self.status, KPGraphStatus::Initialized);
        let mut chain_ref: Option<&Vec<KPGraphChain>> = None;
        for (index, next_chain) in self.filter_chain.iter().enumerate() {
            match chain_ref {
                None => {
                    chain_ref = Some(next_chain);
                    continue;
                }
                Some(prev_chain) => {
                    let prev_chain_outputs: usize = prev_chain.iter().map(|x| { x.filter_context.get_output_count() }).sum();
                    let next_chain_inputs: usize = next_chain.iter().map(|x| { x.filter_context.get_input_count() }).sum();
                    assert_eq!(prev_chain_outputs, next_chain_inputs);
                    if prev_chain_outputs != next_chain_inputs {
                        return Err(anyhow!("mismatch prev chains outputs and next chains inputs. index: {}", index));
                    }

                    // match chain filter
                    let mut next_chain_iter = next_chain.iter();
                    let mut next_chain_item = next_chain_iter.next().unwrap();

                    for prev_chain_item in prev_chain.iter() {
                        let prev_output_count = prev_chain_item.filter_context.get_output_count();
                        let mut prev_output_pad: VecDeque<usize> = (0..prev_output_count).collect();

                        let next_input_count = next_chain_item.filter_context.get_input_count();
                        let mut next_output_pad: VecDeque<usize> = (0..next_input_count).collect();

                        if prev_output_pad.is_empty() {
                            continue;
                        }
                        if next_output_pad.is_empty() {
                            next_chain_item = next_chain_iter.next().unwrap();
                            let next_input_count = next_chain_item.filter_context.get_input_count();
                            next_output_pad = (0..next_input_count).collect();
                        }

                        let prev_pad = prev_output_pad.pop_front().unwrap();
                        let next_pad = next_output_pad.pop_front().unwrap();
                        debug!("link filter prev name: {} index: {}, next name: {}, index: {}",
                                prev_chain_item.filter.get_name(), prev_pad, next_chain_item.filter.get_name(), next_pad);
                        let ret = unsafe { avfilter_link(prev_chain_item.filter_context.get(), prev_pad as c_uint, next_chain_item.filter_context.get(), next_pad as c_uint) };
                        if ret < 0 {
                            return Err(anyhow!("link failed. error: {:?}", averror!(ret)));
                        }
                    }
                }
            }
        }

        // parse config
        let ret = unsafe { avfilter_graph_config(self.filter_graph.get(), ptr::null_mut()) };
        if ret < 0 {
            return Err(anyhow!("parse graph config failed. error: {:?}", averror!(ret)));
        }

        // validate frame_size on audio graph
        if self.media_type == KPAVMediaType::from(AVMEDIA_TYPE_AUDIO) {
            if self.audio_frame_size.is_none() {
                return Err(anyhow!("the frame size can not be none when using an audio graph"));
            }
        }

        self.status = KPGraphStatus::Opened;
        Ok(())
    }

    pub fn stream_to_graph(&mut self, frame: KPAVFrame) -> Result<()> {
        assert!(!frame.is_empty());
        assert_eq!(self.status, KPGraphStatus::Opened);
        trace!("stream to graph. frame pts: {}, media_type: {}", frame.get().pts, self.media_type);

        let source_filter = self.filter_chain.first().unwrap();
        let ret = unsafe { av_buffersrc_add_frame(source_filter.first().unwrap().filter_context.get(), frame.get()) };
        if ret < 0 {
            return Err(anyhow!("stream to graph failed. error: {:?}", averror!(ret)));
        }
        Ok(())
    }

    pub fn stream_from_graph(&mut self) -> Result<()> {
        assert_eq!(self.status, KPGraphStatus::Opened);

        let sink_filter = self.filter_chain.last().unwrap();
        loop {
            let frame = KPAVFrame::new();
            let ret = unsafe { av_buffersink_get_frame(sink_filter.first().unwrap().filter_context.get(), frame.get()) };
            match ret {
                _ if ret >= 0 => {
                    if self.frames.len() >= WARN_QUEUE_LIMIT {
                        warn!("graph context queue length overlong. size: {}, media_type:{}", self.frames.len(), self.media_type);
                    }
                    trace!("stream from graph frame. media_type: {}, pts: {}", self.media_type, frame.get().pts);
                    self.frames.push_back(frame);
                }
                _ if ret == AVERROR(EAGAIN) => {
                    break;
                }
                _ if ret == AVERROR_EOF => {
                    self.status = KPGraphStatus::Ended;
                    break;
                }
                r => {
                    return Err(anyhow!("stream from graph failed. error: {:?}", averror!(r)));
                }
            }
        }
        Ok(())
    }


    pub fn flush(&mut self) -> Result<()> {
        assert_eq!(self.status, KPGraphStatus::Opened);
        let source_filter = self.filter_chain.first().unwrap();
        let ret = unsafe { av_buffersrc_add_frame(source_filter.first().unwrap().filter_context.get(), ptr::null_mut()) };
        if ret < 0 {
            return Err(anyhow!("flush stream to graph failed. error: {:?}", averror!(ret)));
        }
        Ok(())
    }

    pub fn get_status(&self) -> &KPGraphStatus {
        &self.status
    }
}

#[test]
fn test_filter() {
    use crate::decode::decode::KPDecode;

    initialize();
    let mut decode = KPDecode::new(env::var("INPUT_PATH").unwrap());
    decode.open_file().unwrap();

    // set expect stream
    let mut expect_streams = HashMap::new();
    expect_streams.insert(KPAVMediaType::KPAVMEDIA_TYPE_VIDEO, None);
    expect_streams.insert(KPAVMediaType::KPAVMEDIA_TYPE_AUDIO, None);
    decode.set_expect_stream(expect_streams);
    decode.find_streams().unwrap();
    decode.open_codec().unwrap();

    // create graph
    let mut graph = KPGraph::new(&KPAVMediaType::KPAVMEDIA_TYPE_VIDEO);
    graph.injection_source(&decode).unwrap();
    graph.injection_sink().unwrap();

    for get_frame in decode.iter() {
        let (media_type, frame) = get_frame.unwrap();
        info!("decode frame. pts: {}, media_type: {}", frame.get().pts, media_type);

        if media_type == KPAVMediaType::KPAVMEDIA_TYPE_VIDEO {
            graph.stream_to_graph(frame).unwrap();
            for filter_frame in graph.iter() {
                let get_filter_frame = filter_frame.unwrap();
                info!("filter frame. pts: {}", get_filter_frame.get().pts);
            }
        }
    }
    graph.flush().unwrap();
    for filter_frame in graph.iter() {
        let get_filter_frame = filter_frame.unwrap();
        info!("filter frame. pts: {}", get_filter_frame.get().pts);
    }

    assert_eq!(decode.get_status(), &KPCodecStatus::Ended);
    assert_eq!(graph.get_status(), &KPGraphStatus::Ended);
}