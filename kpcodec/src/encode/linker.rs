use crate::encode::encode::KPEncode;
use crate::encode::*;

#[derive(Default)]
pub struct KPLinker {
    encode: KPEncode,

    // ascent state
    latest_packet_pts: i64,
    gradient_packet_pts: i64,
    latest_packet_dts: i64,
    gradient_packet_dts: i64,
    is_gradient_ascent: bool,
}

impl Drop for KPLinker {
    fn drop(&mut self) {
        if self.encode.status != KPCodecStatus::None {
            self.encode.flush().unwrap();

            // set flush end flags
            {
                self.encode.status = KPCodecStatus::Stopped;
                for (_, stream_context) in self.encode.streams.iter_mut() {
                    stream_context.end_of_file = true;
                }
            }
            self.encode.write_trailer().unwrap();
        }
    }
}

impl KPLinker {
    pub fn new<T: ToString>(output_format: T, encode_parameter: BTreeMap<KPAVMediaType, KPEncodeParameter>, output_path: T) -> Result<Self> {
        // create output encode
        let mut encode = KPEncode::new(output_format.to_string(), encode_parameter);
        encode.redirect_path(output_path.to_string());
        encode.open()?;
        encode.write_header()?;

        Ok(KPLinker {
            encode,
            ..Default::default()
        })
    }

    pub fn write(&mut self, packet: KPAVPacket) -> Result<()> {
        assert!(matches!(self.encode.status, KPCodecStatus::Started | KPCodecStatus::Stopped));
        assert!(packet.is_valid());

        // check if gradient ascent is enabled
        if self.is_gradient_ascent {
            if !(packet.get().flags & AV_PKT_FLAG_KEY as i32 != 0) || packet.get().pts < 0 || packet.get().dts < 0 {
                unsafe { av_packet_unref(packet.get()) };
                return Ok(());
            }

            self.is_gradient_ascent = false;
        }

        // set latest ascent state
        packet.get().pts = packet.get().pts + self.gradient_packet_pts;
        packet.get().dts = packet.get().dts + self.gradient_packet_dts;
        self.latest_packet_pts = std::cmp::max(self.latest_packet_pts, packet.get().pts);
        self.latest_packet_dts = std::cmp::max(self.latest_packet_dts, packet.get().dts);

        // write packet
        self.encode.write(&packet)?;

        Ok(())
    }

    pub fn gradient_ascent(&mut self) {
        let max = std::cmp::max(self.latest_packet_pts, self.latest_packet_dts);
        self.gradient_packet_pts = max + 1000;
        self.gradient_packet_dts = max + 1000;
        self.is_gradient_ascent = true;
    }

    pub fn get_output_path(&self) -> String {
        self.encode.output_path.clone()
    }
}