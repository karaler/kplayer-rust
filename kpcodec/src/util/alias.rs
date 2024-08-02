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
        }
    }
}

impl KPAVFormatContext {
    pub fn new() -> Self {
        KPAVFormatContext(unsafe { avformat_alloc_context() })
    }
    pub fn get(&self) -> &mut AVFormatContext {
        if self.0.is_null() {
            panic!("zero pointer");
        }

        unsafe { self.0.as_mut().unwrap() }
    }
    pub fn as_ptr(&self) -> *mut AVFormatContext {
        self.0
    }
}

// KPAVDictionary
pub struct KPAVDictionary {
    pub ptr: *mut *mut AVDictionary,
    pub values: HashMap<String, String>,
}

impl Drop for KPAVDictionary {
    fn drop(&mut self) {
        unsafe { av_dict_free(self.ptr); }
    }
}

impl KPAVDictionary {
    pub fn new<T: ToString>(values: &HashMap<T, T>) -> Self {
        let mut dict: *mut AVDictionary = ptr::null_mut();
        unsafe {
            for (key, value) in values {
                av_dict_set(&mut dict, cstring!(key.to_string()).as_ptr(), cstring!(value.to_string()).as_ptr(), 0);
            }
        }

        KPAVDictionary {
            ptr: &mut dict,
            values: values.iter().map(|(key, value)| (key.to_string(), value.to_string())).collect(),
        }
    }
    pub fn get(&self) -> *mut *mut AVDictionary {
        self.ptr
    }
    pub fn from(ptr: *const AVDictionary) -> HashMap<String, String> {
        let mut map = HashMap::new();
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

// KPAVMediaType
#[derive(Eq, PartialEq, Debug, Hash, Clone)]
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
    pub fn from(media_type: AVMediaType) -> Self {
        KPAVMediaType(media_type)
    }
    pub fn get(&self) -> AVMediaType {
        self.0
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
        if !self.codec_context.is_null() { unsafe { avcodec_free_context(&mut self.codec_context) } }
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
            unsafe { av_frame_free(&mut self.0) }
        }
    }
}

impl KPAVFrame {
    pub fn get(&self) -> &mut AVFrame {
        if self.0.is_null() {
            panic!("zero pointer");
        }
        unsafe { self.0.as_mut().unwrap() }
    }
    pub fn new() -> Self {
        KPAVFrame(unsafe { av_frame_alloc() })
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

impl KPAVRational {
    pub fn from(rational: &AVRational) -> Self {
        KPAVRational(AVRational { num: rational.num, den: rational.den })
    }
    pub fn get(&self) -> AVRational {
        self.0
    }
}
