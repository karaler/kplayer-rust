use std::collections::{BTreeMap, HashMap};
use std::env;
use std::panic::PanicHookInfo;
use crate::util::context::KPAppContext;
use anyhow::{anyhow, Result};
use kpcodec::decode::decode::KPDecode;
use kpcodec::filter::graph::{KPGraph, KPGraphStatus};
use kpcodec::util::alias::KPAVMediaType;
use kpcodec::util::encode_parameter::KPEncodeParameter;
use kpscene::scene::engine::wasm::KPEngine;
use kpscene::scene::scene::{KPScene, KPSceneSortType};
use crate::util::module::resource::ResourceItem;
use std::path::PathBuf;
use log::{debug, info};
use kpcodec::encode::encode::KPEncode;
use kpcodec::encode::linker::KPLinker;
use kpcodec::util::codec_status::KPCodecStatus;
use kpscene::scene::graph::KPSceneGraph;
use crate::init::initialize;
use crate::util::vars::KPAppStatus;

pub struct KPApp {
    context: KPAppContext,
    encode_parameter: BTreeMap<KPAVMediaType, KPEncodeParameter>,
    linker: KPLinker,

    // options
    output_format: String,

    // state
    status: KPAppStatus,
}

impl KPApp {
    pub fn new(context: KPAppContext, encode_parameter: BTreeMap<KPAVMediaType, KPEncodeParameter>) -> Result<Self> {
        let output_format = "flv".to_string();
        let output_path = context.config.output.path.clone();

        let linker = KPLinker::new(output_format.clone(), encode_parameter.clone(), output_path)?;
        Ok(KPApp {
            context,
            encode_parameter,
            output_format,
            status: KPAppStatus::None,
            linker,
        })
    }

    pub async fn start(&mut self) -> Result<()> {
        assert_eq!(self.status, KPAppStatus::None);

        let cfg = self.context.config.clone();

        // start playlist
        for item in cfg.playlist.list.iter() {
            assert!(matches!(self.status, KPAppStatus::None | KPAppStatus::Ended));

            // create decode
            let mut decode = match &item.resource {
                ResourceItem::Single { single } => {
                    let mut decode = KPDecode::new(single.path.clone());
                    decode.set_expect_stream(single.expect_streams.clone());
                    decode
                }
            };
            decode.open().map_err(|err| anyhow!("open input media file failed. path: {:?}, error: {}", item.resource, err))?;
            decode.find_streams()?;
            decode.open_codec()?;
            decode.stream_to_codec()?;
            debug!("create decode success");

            // get decode expect streams
            let expect_streams = decode.get_expect_streams();

            // load scene
            let mut scene = KPScene::new();
            for scene_item in cfg.scene.list.iter() {
                let plugin_path = self.context.plugin_sub_path.join(scene_item.name.clone() + &self.context.plugin_extension);
                debug!("load plugin path: {:?}", plugin_path);
                scene.add_engine(scene_item.name.clone(), KPEngine::new_with_file(plugin_path, scene_item.arguments.clone()).await?);
            }
            debug!("load scene success");

            let mut graph_map = HashMap::new();
            for (media_type, _) in expect_streams.iter() {
                let mut graph = KPGraph::new(media_type);
                graph.injection_source(&decode)?;

                // add before scene
                graph.add_scene(&scene, KPSceneSortType::Before)?;

                // add core
                graph.add_core(media_type, &self.encode_parameter)?;

                // add after scene
                graph.add_scene(&scene, KPSceneSortType::After)?;

                graph.injection_sink()?;
                graph_map.insert(media_type.clone(), graph);
            }

            let mut encode = KPEncode::new("flv", self.encode_parameter.clone());
            encode.enable_sync_timestamp(Some(self.linker.get_output_path()));
            encode.open()?;
            encode.write_header()?;

            // set frame size
            if let Some(audio_graph) = graph_map.get_mut(&KPAVMediaType::KPAVMEDIA_TYPE_AUDIO) {
                audio_graph.set_frame_size(encode.get_audio_frame_size()?)?;
            }

            // transcode
            self.status = KPAppStatus::Initialized;
            self.transcode(decode, graph_map, encode)?;

            // set linker ascent
            self.linker.gradient_ascent();
        }

        self.status = KPAppStatus::Closed;
        Ok(())
    }

    fn transcode(&mut self, mut decode: KPDecode, mut graph_map: HashMap<KPAVMediaType, KPGraph>, mut encode: KPEncode) -> Result<()> {
        assert_eq!(self.status, KPAppStatus::Initialized);

        self.status = KPAppStatus::Starting;
        for get_frame in decode.iter() {
            // process frame
            let (media_type, frame) = get_frame?;
            debug!("decode frame. pts: {}, media_type: {}", frame.get().pts, media_type);

            // send to graph
            let graph = graph_map.get_mut(&media_type).unwrap();
            graph.stream_to_graph(frame)?;
            self.transcode_graph(graph, &mut encode)?;
        }

        // flush graph
        for (_, graph) in graph_map.iter_mut() {
            graph.flush()?;
            self.transcode_graph(graph, &mut encode)?;
        }

        // flush encode
        encode.flush()?;
        self.transcode_encode(&mut encode)?;

        // write encode trailer
        encode.write_trailer()?;

        // validate
        assert_eq!(decode.get_status(), &KPCodecStatus::Ended);
        for (_, graph) in graph_map.iter() {
            assert_eq!(graph.get_status(), &KPGraphStatus::Ended);
        }

        self.status = KPAppStatus::Ended;
        Ok(())
    }

    fn transcode_graph(&mut self, graph: &mut KPGraph, encode: &mut KPEncode) -> Result<()> {
        let media_type = graph.get_media_type().clone();
        for filter_frame in graph.iter() {
            let get_filter_frame = filter_frame?;
            debug!("filter frame. pts: {}", get_filter_frame.get().pts);

            encode.stream_to_encode(get_filter_frame, &media_type)?;

            // send to encode
            self.transcode_encode(encode)?;
        }
        Ok(())
    }

    fn transcode_encode(&mut self, encode: &mut KPEncode) -> Result<()> {
        while let Some(packet) = encode.iter().next() {
            self.linker.write(packet)?;
        }
        Ok(())
    }
}

#[tokio::test]
async fn test_cmd() -> Result<()> {
    initialize();
    let home_path = PathBuf::from(env::var("HOME_PATH")?);
    let config_path = home_path.join("kplayer.json");

    let context = KPAppContext::new(home_path, config_path)?;
    let mut encode_parameter = BTreeMap::new();
    encode_parameter.insert(KPAVMediaType::KPAVMEDIA_TYPE_VIDEO, KPEncodeParameter::default(&KPAVMediaType::KPAVMEDIA_TYPE_VIDEO));
    encode_parameter.insert(KPAVMediaType::KPAVMEDIA_TYPE_AUDIO, KPEncodeParameter::default(&KPAVMediaType::KPAVMEDIA_TYPE_AUDIO));
    let mut app = KPApp::new(context, encode_parameter)?;
    app.start().await?;
    Ok(())
}