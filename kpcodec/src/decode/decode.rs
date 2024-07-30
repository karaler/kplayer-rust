use std::env;
use log::info;
use crate::decode::*;

#[derive(Default)]
pub struct KPDecodeStreamContext {
    media_type: AVMediaType,
    codec_context_ptr: KPCodecContextPtr,
    end_of_file: bool,
    metadata: HashMap<String, String>,
}

#[derive(Default)]
pub struct KPDecode {
    input_path: String,

    // formation
    format_context_options: HashMap<String, String>,
    format_context_ptr: KPFormatContextPtr,

    // open options
    open_timeout: u32,
    start_point: Option<u64>,
    end_point: Option<u64>,
    expect_stream_index: HashMap<AVMediaType, u32>,
    encode_hardware: bool,

    // media information
    format_name: String,
    metadata: HashMap<String, String>,
    streams: HashMap<u32, KPDecodeStreamContext>,
    start_time: Duration,
    duration: Duration,
    bit_rate: u64,

    // state
    status: KPCodecStatus,
    position: Duration,
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
            ..Default::default()
        }
    }

    pub fn open(&mut self) -> Result<()> {
        // open file
        self.format_context_ptr = KPFormatContextPtr::new();
        let filepath: CString = cstring!(self.input_path.clone());
        let open_options = KPAVDictionary::new(&self.format_context_options);
        if unsafe {
            avformat_open_input(&mut self.format_context_ptr.0, filepath.as_ptr(), ptr::null_mut(), open_options.get())
        } != 0 { return Err(anyhow!("open input failed")); }

        // read information
        let format_context = self.format_context_ptr.get();
        self.format_name = cstr!((*format_context.iformat).long_name).to_string();
        self.start_time = Duration::from_micros({
            if format_context.start_time == AV_NOPTS_VALUE { 0 } else {
                format_context.start_time as u64
            }
        });
        self.duration = Duration::from_micros(format_context.duration as u64);
        self.bit_rate = format_context.bit_rate as u64;

        info!("open file success. path:{}, format_name:{}, start_time:{:?}, duration:{:?}, bit_rate:{}",
            self.input_path, self.format_name,self.start_time,self.duration,self.bit_rate);

        Ok(())
    }

    pub fn match_streams(&mut self) -> Result<()> {
        Ok(())
    }
}

#[test]
fn open_file() {
    initialize();
    let mut decode = KPDecode::new(env::var("INPUT_PATH").unwrap());
    decode.open().unwrap();
}

