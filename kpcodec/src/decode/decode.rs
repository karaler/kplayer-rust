use std::ffi::c_char;
use std::slice::Iter;
use crate::decode::*;
use crate::filter::graph_source::{KPGraphSourceAttribute, KPGraphSourceRely};
use crate::util::encode_parameter::{KPEncodeParameter, KPEncodeParameterPreset, KPEncodeParameterProfile};

#[derive(Default, Debug)]
pub struct KPDecodeStreamContext {
    media_type: KPAVMediaType,
    time_base: KPAVRational,
    codec_context_ptr: KPAVCodecContext,
    end_of_file: bool,
    metadata: BTreeMap<String, String>,
}

#[derive(Default, Debug)]
pub struct KPDecode {
    input_path: String,

    // formation
    format_context_options: HashMap<String, String>,
    format_context_ptr: KPAVFormatContext,

    // open options
    open_timeout: usize,
    start_point: Option<Duration>,
    end_point: Option<Duration>,
    expect_stream_index: HashMap<KPAVMediaType, Option<usize>>,
    encode_hardware: bool,
    enable_loop: bool,
    seek: usize,
    end: usize,

    // media information
    format_name: String,
    metadata: HashMap<String, String>,
    streams: BTreeMap<usize, KPDecodeStreamContext>,
    start_time: Duration,
    duration: Duration,
    bit_rate: u64,

    // state
    pub(super) status: KPCodecStatus,
    pub(super) position: Duration,
    lead_stream_index: Option<usize>,
    enable_loop_count: usize,
    loop_latest_pts: i64,
    loop_latest_dts: i64,
    loop_gradient_pts: i64,
    loop_gradient_dts: i64,

    // cache
    packet: KPAVPacket,
}

pub struct KPDecodeIterator<'a> {
    decode: &'a mut KPDecode,
}

impl<'a> Iterator for KPDecodeIterator<'a> {
    type Item = Result<(KPAVMediaType, KPAVFrame)>;

    fn next(&mut self) -> Option<Self::Item> {
        let decode = &mut self.decode;

        // stream to queue
        while decode.status == KPCodecStatus::Started {
            if let Err(err) = decode.stream_to_codec() { return Some(Err(err)); }
            return match decode.stream_from_codec() {
                Ok(f) => match f {
                    None => continue,
                    Some(frame) => Some(Ok(frame))
                },
                Err(err) => Some(Err(err))
            };
        }
        None
    }
}

impl KPGraphSourceRely for KPDecode {
    fn get_source(&self, media_type: &KPAVMediaType) -> Result<KPGraphSourceAttribute> {
        assert_eq!(self.status, KPCodecStatus::Started);
        let get_stream_index = |media_type: &KPAVMediaType| {
            match self.expect_stream_index.get(media_type) {
                None => Err(anyhow!("not exist except stream. media_type: {}", media_type)),
                Some(i) => Ok(i.unwrap()),
            }
        };
        match media_type.clone() {
            m if m == KPAVMediaType::KPAVMEDIA_TYPE_VIDEO => {
                let video_stream_context = self.streams.get(&get_stream_index(&KPAVMediaType::KPAVMEDIA_TYPE_VIDEO)?).unwrap();
                let codec_context = video_stream_context.codec_context_ptr.get();
                Ok(KPGraphSourceAttribute::Video {
                    width: codec_context.width as usize,
                    height: codec_context.height as usize,
                    pix_fmt: KPAVPixelFormat::from(codec_context.pix_fmt),
                    time_base: video_stream_context.time_base.clone(),
                    frame_rate: KPAVRational::from(codec_context.framerate),
                    pixel_aspect: KPAVRational::from(codec_context.sample_aspect_ratio),
                })
            }
            m if m == KPAVMediaType::KPAVMEDIA_TYPE_AUDIO => {
                let audio_stream_context = self.streams.get(&get_stream_index(&KPAVMediaType::KPAVMEDIA_TYPE_AUDIO)?).unwrap();
                let codec_context = audio_stream_context.codec_context_ptr.get();
                Ok(KPGraphSourceAttribute::Audio {
                    sample_rate: codec_context.sample_rate as usize,
                    sample_fmt: KPAVSampleFormat::from(codec_context.sample_fmt),
                    channel_layout: codec_context.channel_layout as usize,
                    channels: codec_context.channels as usize,
                    time_base: audio_stream_context.time_base.clone(),
                })
            }
            m => {
                Err(anyhow!("not support media type. media_type: {}", m))
            }
        }
    }
}

impl KPDecode {
    pub fn new<T: ToString>(input_path: T) -> Self {
        let open_timeout = 10;
        let mut format_context_options = HashMap::new();
        format_context_options.insert(String::from("scan_all_pmts"), String::from("1"));
        format_context_options.insert(String::from("rw_timeout"), String::from(open_timeout.to_string()));

        KPDecode {
            input_path: input_path.to_string(),
            format_context_options,
            open_timeout,
            packet: KPAVPacket::new(),
            enable_loop: false,
            ..Default::default()
        }
    }

    // set flag
    pub fn set_expect_stream(&mut self, expect_streams: HashMap<KPAVMediaType, Option<usize>>) -> &mut Self {
        self.expect_stream_index = expect_streams;
        self
    }

    // operate
    pub fn open(&mut self) -> Result<()> {
        assert_eq!(self.status, KPCodecStatus::None);

        // open file
        {
            let mut open_options = KPAVDictionary::new(&self.format_context_options);
            let mut open_options_ptr = open_options.get();

            let mut format_context_ptr: *mut AVFormatContext = ptr::null_mut();
            let filepath: CString = cstring!(self.input_path.clone());
            let ret = unsafe {
                avformat_open_input(&mut format_context_ptr, filepath.as_ptr(), ptr::null_mut(), &mut open_options_ptr)
            };
            if ret < 0 { return Err(anyhow!("open input failed. error: {:?}",  averror!(ret))); }
            open_options.set(open_options_ptr);
            self.format_context_ptr = KPAVFormatContext::from(format_context_ptr);

            assert_eq!(open_options_ptr, open_options.get());
            assert_eq!(format_context_ptr, self.format_context_ptr.as_ptr());
        }

        // read information
        let format_context = self.format_context_ptr.get();
        self.format_name = cstr!((*format_context.iformat).long_name);
        self.start_time = Duration::from_micros({
            if format_context.start_time == AV_NOPTS_VALUE { 0 } else {
                format_context.start_time as u64
            }
        });
        self.duration = Duration::from_micros(format_context.duration as u64);
        self.bit_rate = format_context.bit_rate as u64;

        self.status = KPCodecStatus::Opened;
        info!("open file success. path:{}, format_name:{}, start_time:{:?}, duration:{:?}, bit_rate:{}",
            self.input_path, self.format_name,self.start_time,self.duration,self.bit_rate);
        Ok(())
    }

    pub fn find_streams(&mut self) -> Result<()> {
        assert_eq!(self.status, KPCodecStatus::Opened);
        let ret = unsafe { avformat_find_stream_info(self.format_context_ptr.as_ptr(), ptr::null_mut()) };
        if ret < 0 { return Err(anyhow!("find streams failed. error: {:?}", averror!(ret))); }

        let format_context = self.format_context_ptr.get();
        // fill in stream
        unsafe {
            for i in 0..format_context.nb_streams as usize {
                let stream_ptr_ptr = format_context.streams.add(i);
                let stream_ptr = *stream_ptr_ptr;
                if stream_ptr.is_null() {
                    return Err(anyhow!("find stream failed, stream is null. index: {}", i));
                }

                let stream = *stream_ptr;
                let metadata = KPAVDictionary::from(stream.metadata);
                let codec_context = KPDecodeStreamContext {
                    media_type: KPAVMediaType::from((*stream.codecpar).codec_type),
                    time_base: KPAVRational::from(stream.time_base),
                    codec_context_ptr: Default::default(),
                    end_of_file: false,
                    metadata,
                };

                self.streams.insert(i, codec_context);
            }
        }
        debug!("find streams: {:?}", self.streams);

        if self.expect_stream_index.is_empty() {
            debug!("expect stream is empty");

            // gather media types of all streams
            unsafe {
                for i in 0..self.format_context_ptr.get().nb_streams as usize {
                    let stream_ptr_ptr = self.format_context_ptr.get().streams.add(i);
                    let stream_ptr = *stream_ptr_ptr;
                    if !stream_ptr.is_null() {
                        let stream = *stream_ptr;
                        let media_type = KPAVMediaType::from((*stream.codecpar).codec_type);
                        self.expect_stream_index.insert(media_type, None);
                    }
                }
            }
        }

        // set expect stream
        for (media_type, stream_index_opt) in self.expect_stream_index.iter_mut() {
            let stream_index: i64 = match stream_index_opt {
                None => -1,
                Some(s) => s.clone() as i64,
            };

            // find stream
            let ret = unsafe { av_find_best_stream(self.format_context_ptr.as_ptr(), media_type.get(), stream_index as c_int, -1 as c_int, ptr::null_mut(), 0 as c_int) };
            if ret < 0 {
                return Err(anyhow!("find expect stream failed. media_type:{}, stream_index: {:?}, error: {:?}", media_type,stream_index_opt,averror!(ret)));
            }
            *stream_index_opt = Some(ret as usize);
        }

        debug!("expect streams: {:?}",self.expect_stream_index);

        Ok(())
    }

    pub fn open_codec(&mut self) -> Result<()> {
        assert_eq!(self.status, KPCodecStatus::Opened);
        assert!(!self.streams.is_empty());
        for (media_type, stream_index_opt) in self.expect_stream_index.iter() {
            let format_context = unsafe { (*self.format_context_ptr.get()) };
            let stream_index = stream_index_opt.unwrap();
            let stream_ptr = unsafe {
                let streams_ptr_ptr = format_context.streams.add(stream_index);
                *streams_ptr_ptr
            };
            if stream_ptr.is_null() {
                return Err(anyhow!("get stream failed, stream is null. index:{}", stream_index));
            }
            let stream = unsafe { *stream_ptr };

            // find codec
            let codec = unsafe { avcodec_find_decoder((*stream.codecpar).codec_id) };
            if codec.is_null() {
                return Err(anyhow!("decoder not found. index:{}, codec_id:{}", stream_index, unsafe{(*stream.codecpar).codec_id} ));
            }

            // open codec
            let stream_context = self.streams.get_mut(&stream_index).unwrap();
            assert!(stream_context.codec_context_ptr.is_null());

            let codec_context = KPAVCodecContext::new(codec);
            let ret = unsafe { avcodec_parameters_to_context(codec_context.get(), stream.codecpar) };
            if ret < 0 {
                return Err(anyhow!("set parameters to codec failed. index:{}, error: {:?}", stream_index, averror!(ret)));
            }
            let ret = unsafe { avcodec_open2(codec_context.get(), codec, ptr::null_mut()) };
            if ret < 0 {
                return Err(anyhow!("open codec failed. index:{}, error: {:?}", stream_index, averror!(ret)));
            }
            debug!("open codec success. media_type:{}, index:{}, codec name:{}", media_type, stream_index, cstr!((*codec).long_name));
            stream_context.codec_context_ptr = codec_context;
        }

        self.status = KPCodecStatus::Started;

        // initial state
        self.set_point()?;

        Ok(())
    }

    // 1). set lead stream index
    // 2). set start time point
    fn set_point(&mut self) -> Result<()> {
        assert!(matches!(self.status, KPCodecStatus::Opened|KPCodecStatus::Started));
        let lead_stream_index = match self.expect_stream_index.get(&KPAVMediaType::from(AVMEDIA_TYPE_VIDEO)) {
            None => self.expect_stream_index.iter().next().unwrap().1.unwrap(),
            Some(video_stream_index) => {
                video_stream_index.unwrap()
            }
        };
        self.lead_stream_index = Some(lead_stream_index);

        // set start point
        if let Some(start_point) = self.start_point {
            let stream = self.streams.get(&lead_stream_index).unwrap();
            let seek_timestamp = unsafe { av_rescale_q(start_point.as_micros() as i64, AV_TIME_BASE_Q, stream.time_base.get()) };
            let ret = unsafe { av_seek_frame(self.format_context_ptr.get(), lead_stream_index as c_int, seek_timestamp, AVSEEK_FLAG_BACKWARD as c_int) };
            if ret < 0 {
                return Err(anyhow!("seek start point failed. error:{:?}", averror!(ret)));
            }
        }

        // flush codec
        for (_, stream_context) in self.streams.iter() {
            if !stream_context.codec_context_ptr.is_null() {
                unsafe { avcodec_flush_buffers(stream_context.codec_context_ptr.get()) };
            }
        }
        Ok(())
    }

    pub fn stream_to_codec(&mut self) -> Result<()> {
        assert_eq!(self.status, KPCodecStatus::Started);
        assert!(self.lead_stream_index.is_some());
        assert!((*self.packet.get()).buf.is_null());
        let lead_stream_index = self.lead_stream_index.unwrap();

        // read a packet
        let ret = unsafe { av_read_frame(self.format_context_ptr.get(), self.packet.get()) };
        if ret < 0 {
            return match ret {
                AVERROR_EOF => {
                    // enable loop
                    if self.enable_loop {
                        if self.start_point.is_none() {
                            self.start_point = Some(Duration::from_secs(0));
                        }
                        self.set_point()?;

                        self.enable_loop_count += 1;
                        self.loop_gradient_pts = std::cmp::max(self.loop_latest_pts, self.loop_latest_dts);
                        self.loop_gradient_dts = std::cmp::max(self.loop_latest_pts, self.loop_latest_dts);
                        return Ok(());
                    }

                    // set eof
                    self.status = KPCodecStatus::Ended;
                    for (_, expect_stream_index) in self.expect_stream_index.iter() {
                        let stream_context = self.streams.get_mut(&expect_stream_index.unwrap()).unwrap();
                        stream_context.codec_context_ptr.flush()?;
                    }
                    Ok(())
                }
                _ => { Err(anyhow!("stream packet failed. error: {:?}", averror!(ret))) }
            };
        }
        let packet = self.packet.get();

        // validate packet
        if !self.expect_stream_index.values().any(|&value| value == Some(packet.stream_index as usize)) {
            debug!("not expect stream packet, pts:{}, dts:{}, index:{}", packet.pts, packet.dts, packet.stream_index);
            self.packet.clean();
            return Ok(());
        }
        if packet.pts == AV_NOPTS_VALUE {
            debug!("skip invalid packet, pts:{}, dts:{}, index:{}", packet.pts, packet.dts, packet.stream_index);
            self.packet.clean();
            return Ok(());
        }

        packet.pts += self.loop_gradient_pts;
        packet.dts += self.loop_gradient_dts;
        self.loop_latest_pts = packet.pts;
        self.loop_latest_dts = packet.dts;

        // set state
        if packet.stream_index as usize == lead_stream_index {
            let stream_context = self.streams.get(&self.lead_stream_index.unwrap()).unwrap();
            self.position = Duration::from_secs_f64(av_q2d(stream_context.time_base.get()) as f64 * packet.pts as f64 + self.start_time.as_micros() as f64);
        }

        // compare end_point
        if let Some(end_point) = self.end_point {
            if self.position > end_point {
                self.status = KPCodecStatus::Ended;
                return Ok(());
            }
        }

        // send to codec
        let stream_context = self.streams.get(&(packet.stream_index as usize)).unwrap();
        trace!("send packet to codec. position: {:?}, index:{}, media_type:{}, pts:{}, dts:{}, size:{}",self.position,self.packet.get().stream_index,stream_context.media_type,self.packet.get().pts,self.packet.get().dts, self.packet.get().size);

        assert!(!stream_context.codec_context_ptr.is_flushed());
        let ret = unsafe { avcodec_send_packet(stream_context.codec_context_ptr.get(), self.packet.get()) };
        if ret < 0 {
            return Err(anyhow!("send packet to codec failed. error:{:?}",averror!(ret)));
        }

        self.packet.clean();
        Ok(())
    }

    pub fn stream_from_codec(&mut self) -> Result<Option<(KPAVMediaType, KPAVFrame)>> {
        assert!(matches!(self.status, KPCodecStatus::Started|KPCodecStatus::Ended));
        if !self.expect_stream_index.iter().any(|(_, v)| {
            let stream_context = self.streams.get(&v.unwrap()).unwrap();
            !stream_context.end_of_file
        }) {
            self.status = KPCodecStatus::Stopped;
            return Ok(None);
        }

        // receive expect stream context
        for (media_type, expect_stream_index) in self.expect_stream_index.iter() {
            let stream_index = expect_stream_index.unwrap();
            let stream_context = self.streams.get_mut(&stream_index).unwrap();
            assert_eq!(&stream_context.media_type, media_type);

            if stream_context.end_of_file { continue; }
            // get frame
            loop {
                let frame = KPAVFrame::new();
                let ret = unsafe { avcodec_receive_frame(stream_context.codec_context_ptr.get(), frame.get()) };
                match ret {
                    _ if ret >= 0 => {
                        trace!("receipt frame. index:{}, media_type:{}, pts:{}", stream_index, media_type, frame.get().pts);
                        return Ok(Some((media_type.clone(), frame)));
                    }
                    _ if ret == AVERROR(EAGAIN) => {
                        break;
                    }
                    _ if ret == AVERROR_EOF => {
                        stream_context.end_of_file = true;
                        break;
                    }
                    r => {
                        return Err(anyhow!("receipt from codec failed. index:{}, error:{:?}", stream_index, averror!(r)));
                    }
                }
            }
        }
        Ok(None)
    }

    pub fn get_status(&self) -> &KPCodecStatus {
        &self.status
    }

    pub fn get_expect_streams(&self) -> &HashMap<KPAVMediaType, Option<usize>> {
        &self.expect_stream_index
    }

    pub fn set_enable_loop(&mut self, enable: bool) {
        self.enable_loop = enable;
    }
}

impl KPDecode {
    pub fn iter(&mut self) -> KPDecodeIterator {
        KPDecodeIterator {
            decode: self,
        }
    }
}

#[test]
fn open_file() {
    initialize();
    let mut decode = KPDecode::new(env::var("INPUT_SHORT_PATH").unwrap());
    decode.open().unwrap();

    // set expect stream
    let mut expect_streams = HashMap::new();
    expect_streams.insert(KPAVMediaType::from(AVMEDIA_TYPE_VIDEO), None);
    expect_streams.insert(KPAVMediaType::from(AVMEDIA_TYPE_AUDIO), None);
    decode.set_expect_stream(expect_streams);
    decode.find_streams().unwrap();
    decode.open_codec().unwrap();

    for get_frame in decode.iter() {
        let (media_type, frame) = get_frame.unwrap();
        info!("get frame. {:?}, meida_type: {}", frame, media_type);
    }
}

#[test]
fn open_invalid_file() {
    initialize();
    let mut decode = KPDecode::new(env::var("INPUT_INVALID_PATH").unwrap());
    assert!(decode.open().is_err());
}

#[test]
fn decode_loop() {
    initialize();
    let mut decode = KPDecode::new(env::var("INPUT_SHORT_PATH").unwrap());
    decode.open().unwrap();

    // set expect stream
    let mut expect_streams = HashMap::new();
    expect_streams.insert(KPAVMediaType::from(AVMEDIA_TYPE_VIDEO), None);
    expect_streams.insert(KPAVMediaType::from(AVMEDIA_TYPE_AUDIO), None);
    decode.set_expect_stream(expect_streams);
    decode.find_streams().unwrap();
    decode.open_codec().unwrap();
    decode.set_enable_loop(true);

    for get_frame in decode.iter() {
        let (media_type, frame) = get_frame.unwrap();
        info!("get frame. {:?}, meida_type: {}", frame, media_type);
    }
}