use std::ffi::CString;
use std::ptr::null_mut;
use crate::init::initialize;
use crate::mut_ptr;
use crate::util::*;

// KPAVFormatContext
#[derive(Debug)]
pub struct KPAVFormatContext(pub *mut AVFormatContext);

impl Default for KPAVFormatContext {
    fn default() -> Self {
        KPAVFormatContext(ptr::null_mut())
    }
}

impl Drop for KPAVFormatContext {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe { avformat_free_context(self.0); }
            self.0 = ptr::null_mut();
        }
    }
}

impl KPAVFormatContext {
    pub fn new() -> Self {
        KPAVFormatContext(unsafe { avformat_alloc_context() })
    }

    pub fn from(format_context: *mut AVFormatContext) -> Self {
        KPAVFormatContext(format_context)
    }

    pub fn get(&self) -> &mut AVFormatContext {
        if self.0.is_null() {
            panic!("zero pointer");
        }

        unsafe { self.0.as_mut().unwrap() }
    }

    pub fn set(&mut self, format_context: *mut AVFormatContext) {
        self.0 = format_context;
    }

    pub fn as_ptr(&self) -> *mut AVFormatContext {
        self.0
    }

    pub fn is_null(&self) -> bool {
        self.0.is_null()
    }
}

// KPAVDictionary
pub struct KPAVDictionary {
    pub ptr: *mut AVDictionary,
    pub values: HashMap<String, String>,
}

impl Drop for KPAVDictionary {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { av_dict_free(&mut self.ptr); }
            self.ptr = ptr::null_mut();
        }
    }
}

impl KPAVDictionary {
    pub fn new<T: ToString>(values: &HashMap<T, T>) -> Self {
        let mut dict: *mut AVDictionary = ptr::null_mut();
        for (key, value) in values {
            let key_cstring = cstring!(key.to_string());
            let value_cstring = cstring!(value.to_string());
            unsafe { av_dict_set(&mut dict, key_cstring.as_ptr(), value_cstring.as_ptr(), 0) };
        }

        KPAVDictionary {
            ptr: dict,
            values: values.iter().map(|(key, value)| (key.to_string(), value.to_string())).collect(),
        }
    }

    pub fn get(&self) -> *mut AVDictionary {
        self.ptr
    }

    pub fn set(&mut self, ptr: *mut AVDictionary) {
        self.ptr = ptr;
    }

    pub fn from(ptr: *const AVDictionary) -> BTreeMap<String, String> {
        let mut map = BTreeMap::new();
        unsafe {
            let mut tag: *mut AVDictionaryEntry = ptr::null_mut();
            tag = av_dict_get(ptr, cstring!("").as_ptr(), tag, AV_DICT_IGNORE_SUFFIX as c_int);
            while !tag.is_null() {
                let key = (*tag).key;
                let value = (*tag).value;
                map.insert(cstr!(key), cstr!(value));

                tag = av_dict_get(ptr, cstring!("").as_ptr(), tag, AV_DICT_IGNORE_SUFFIX as c_int);
            }
        }

        map
    }
}

#[test]
fn test_dict() {
    initialize();
    let mut map: HashMap<String, String> = HashMap::new();
    map.insert("hello".to_string(), "world".to_string());
    let dict = KPAVDictionary::new(&map);
}

// KPAVMediaType
#[derive(Eq, PartialEq, Debug, Hash, Copy, Clone,Ord, PartialOrd)]
pub struct KPAVMediaType(AVMediaType);

impl Display for KPAVMediaType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let media_type_str = unsafe { av_get_media_type_string(self.0) };
        write!(f, "{}", cstr!(media_type_str))
    }
}

impl Default for KPAVMediaType {
    fn default() -> Self {
        KPAVMediaType(AVMEDIA_TYPE_UNKNOWN)
    }
}

impl KPAVMediaType {
    pub const KPAVMEDIA_TYPE_VIDEO: KPAVMediaType = KPAVMediaType(AVMEDIA_TYPE_VIDEO);
    pub const KPAVMEDIA_TYPE_AUDIO: KPAVMediaType = KPAVMediaType(AVMEDIA_TYPE_AUDIO);

    pub fn from(media_type: AVMediaType) -> Self {
        KPAVMediaType(media_type)
    }

    pub fn get(&self) -> AVMediaType {
        self.0
    }

    pub fn is_unknown(&self) -> bool {
        self.0 == AVMEDIA_TYPE_UNKNOWN
    }

    pub fn is_video(&self) -> bool {
        self.0 == AVMEDIA_TYPE_VIDEO
    }

    pub fn is_audio(&self) -> bool {
        self.0 == AVMEDIA_TYPE_AUDIO
    }
}

// KPAVCodecContext
#[derive(Debug)]
pub struct KPAVCodecContext {
    codec_context: *mut AVCodecContext,
    is_flushed: bool,
}

impl Default for KPAVCodecContext {
    fn default() -> Self {
        KPAVCodecContext {
            codec_context: ptr::null_mut(),
            is_flushed: false,
        }
    }
}

impl Drop for KPAVCodecContext {
    fn drop(&mut self) {
        if !self.codec_context.is_null() {
            unsafe { avcodec_free_context(&mut self.codec_context) };
            self.codec_context = ptr::null_mut();
        }
    }
}

impl KPAVCodecContext {
    pub fn new(codec: *const AVCodec) -> Self {
        let codec_context = unsafe { avcodec_alloc_context3(codec) };
        if codec_context.is_null() {
            panic!("alloc codec context failed");
        }
        KPAVCodecContext { codec_context, is_flushed: false }
    }

    pub fn get(&self) -> &mut AVCodecContext {
        if self.codec_context.is_null() {
            panic!("zero pointer");
        }
        unsafe { self.codec_context.as_mut().unwrap() }
    }

    pub fn is_null(&self) -> bool {
        self.codec_context.is_null()
    }

    pub fn is_flushed(&self) -> bool {
        self.is_flushed
    }

    pub fn flush(&mut self) -> Result<()> {
        assert!(!self.get().codec.is_null());
        if unsafe { av_codec_is_decoder(self.get().codec) } != 0 {
            let ret = unsafe { avcodec_send_packet(self.get(), ptr::null_mut()) };
            if ret < 0 {
                return Err(anyhow!("flush codec failed. error: {:?}", averror!(ret)));
            }
        } else if unsafe { av_codec_is_encoder(self.get().codec) } != 0 {
            let ret = unsafe { avcodec_send_frame(self.get(), ptr::null_mut()) };
            if ret < 0 {
                return Err(anyhow!("flush codec failed. error: {:?}", averror!(ret)));
            }
        } else {
            return Err(anyhow!("invalid codec type"));
        }
        info!("flush codec context success");
        self.is_flushed = true;
        Ok(())
    }
}

// KPAVFrame
#[derive(Debug)]
pub struct KPAVFrame(*mut AVFrame);

impl Default for KPAVFrame {
    fn default() -> Self {
        KPAVFrame(ptr::null_mut())
    }
}

impl Drop for KPAVFrame {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe {
                av_frame_free(&mut self.0);
                self.0 = ptr::null_mut();
            }
        }
    }
}

impl KPAVFrame {
    pub fn new() -> Self {
        KPAVFrame(unsafe { av_frame_alloc() })
    }

    pub fn get(&self) -> &mut AVFrame {
        if self.0.is_null() {
            panic!("zero pointer");
        }
        unsafe { self.0.as_mut().unwrap() }
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_null() || self.get().buf.is_empty()
    }
}

// KPAVPacket
#[derive(Debug)]
pub struct KPAVPacket(*mut AVPacket);

impl Default for KPAVPacket {
    fn default() -> Self {
        KPAVPacket(ptr::null_mut())
    }
}

impl KPAVPacket {
    pub fn get(&self) -> &mut AVPacket {
        if self.0.is_null() {
            panic!("zero pointer");
        }
        unsafe { self.0.as_mut().unwrap() }
    }

    pub fn new() -> Self {
        KPAVPacket(unsafe { av_packet_alloc() })
    }

    pub fn clean(&self) {
        unsafe { av_packet_unref(self.0) }
    }

    pub fn is_valid(&self) -> bool {
        !self.0.is_null() && self.get().pts != AV_NOPTS_VALUE
    }
}

// KPAVRational
#[derive(Debug, Clone)]
pub struct KPAVRational(AVRational);

impl Default for KPAVRational {
    fn default() -> Self {
        let rational = AVRational { num: 0, den: 0 };
        KPAVRational(rational)
    }
}

impl Display for KPAVRational {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", format!("{}/{}", self.0.num, self.0.den))
    }
}

impl KPAVRational {
    pub const fn from(rational: AVRational) -> Self {
        KPAVRational(rational)
    }
    pub const fn from_fps(fps: usize) -> Self {
        KPAVRational(AVRational { num: fps as c_int, den: 1 })
    }

    pub fn get(&self) -> AVRational {
        self.0
    }

    pub fn is_empty(&self) -> bool {
        self.0.num == 0 && self.0.den == 1
    }
}

// KPAVFilterGraph
pub struct KPAVFilterGraph {
    filter_graph: *mut AVFilterGraph,
}

impl Default for KPAVFilterGraph {
    fn default() -> Self {
        KPAVFilterGraph { filter_graph: ptr::null_mut() }
    }
}

impl Drop for KPAVFilterGraph {
    fn drop(&mut self) {
        if !self.filter_graph.is_null() {
            unsafe {
                avfilter_graph_free(&mut self.filter_graph);
                self.filter_graph = ptr::null_mut();
            }
        }
    }
}

impl KPAVFilterGraph {
    pub fn new() -> Self {
        KPAVFilterGraph {
            filter_graph: unsafe { avfilter_graph_alloc() }
        }
    }

    pub fn as_ptr(&self) -> *mut AVFilterGraph {
        assert!(!self.filter_graph.is_null());
        self.filter_graph
    }

    pub fn get(&self) -> &mut AVFilterGraph {
        assert!(!self.filter_graph.is_null());
        if self.filter_graph.is_null() {
            panic!("zero pointer");
        }

        unsafe { self.filter_graph.as_mut().unwrap() }
    }
}

// KPFilter
#[derive(Clone)]
pub struct KPAVFilter(*const AVFilter);

impl Default for KPAVFilter {
    fn default() -> Self {
        KPAVFilter(ptr::null_mut())
    }
}

impl KPAVFilter {
    pub fn as_ptr(&self) -> *const AVFilter {
        self.0
    }

    pub fn from(filter: *const AVFilter) -> Self {
        KPAVFilter(filter)
    }

    pub fn is_null(&self) -> bool {
        self.0.is_null()
    }
}

// KPFilterContext
pub struct KPAVFilterContext {
    filter_context: *mut AVFilterContext,
}

impl Default for KPAVFilterContext {
    fn default() -> Self {
        KPAVFilterContext { filter_context: ptr::null_mut() }
    }
}

impl KPAVFilterContext {
    pub fn get(&self) -> &mut AVFilterContext {
        assert!(!self.filter_context.is_null());
        if self.filter_context.is_null() {
            panic!("zero pointer");
        }
        unsafe { self.filter_context.as_mut().unwrap() }
    }

    pub fn from(filter_context: *mut AVFilterContext) -> Self {
        KPAVFilterContext { filter_context }
    }

    pub fn as_ptr(&self) -> *mut AVFilterContext {
        self.filter_context
    }

    pub fn get_input_count(&self) -> usize {
        self.get().nb_inputs as usize
    }

    pub fn get_output_count(&self) -> usize {
        self.get().nb_outputs as usize
    }

    pub fn is_null(&self) -> bool {
        self.filter_context.is_null()
    }
}

// KPAVPixelFormat
#[derive(Debug)]
pub struct KPAVPixelFormat(AVPixelFormat);

impl Display for KPAVPixelFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = unsafe { av_get_pix_fmt_name(self.0) };
        write!(f, "{}", format!("{}", cstr!(name)))
    }
}

impl KPAVPixelFormat {
    pub fn as_str(&self) -> String {
        format!("{}", self.0)
    }

    pub const fn from(pix_fmt: AVPixelFormat) -> Self {
        KPAVPixelFormat(pix_fmt)
    }

    pub fn get(&self) -> AVPixelFormat {
        self.0
    }
}

// KPAVSampleFormat
#[derive(Debug)]
pub struct KPAVSampleFormat(AVSampleFormat);

impl Display for KPAVSampleFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = unsafe { av_get_sample_fmt_name(self.0) };
        write!(f, "{}", cstr!(name))
    }
}

impl KPAVSampleFormat {
    pub fn as_str(&self) -> String {
        format!("{}", self.0.to_string())
    }

    pub const fn from(sample_format: AVSampleFormat) -> Self {
        KPAVSampleFormat(sample_format)
    }

    pub fn get(&self) -> AVSampleFormat {
        self.0
    }
}

// KPAVCodecId
#[derive(Eq, PartialEq, Debug, Default)]
pub struct KPAVCodecId(AVCodecID);

impl Display for KPAVCodecId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let name = unsafe { avcodec_get_name(self.0) };
        write!(f, "{}", cstr!(name))
    }
}

impl KPAVCodecId {
    pub const fn from(codec_id: AVCodecID) -> Self {
        KPAVCodecId(codec_id)
    }
    pub fn get(&self) -> AVCodecID {
        self.0
    }
    pub fn is_none(&self) -> bool {
        self.0 == AV_CODEC_ID_NONE
    }
}