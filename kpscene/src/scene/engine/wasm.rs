use std::sync::Arc;
use derivative::Derivative;
use wasmtime::{Config, Engine, Memory};
use wasmtime::Module;
use wasmtime_wasi::preview1::WasiP1Ctx;
use wasmtime_wasi::WasiCtxBuilder;
use crate::scene::engine::*;
use crate::scene::scene::KPSceneSortType;

const DEFAULT_MODULE: &str = "";
const DEFAULT_MEMORY: &str = "memory";

#[derive(Derivative)]
#[derivative(Debug)]
pub struct KPEngine {
    #[derivative(Debug = "ignore")]
    pub(crate) bytecode: Vec<u8>,
    pub app: String,
    pub author: String,
    pub media_type: KPAVMediaType,
    pub sort_type: KPSceneSortType,
    pub default_arguments: BTreeMap<String, String>,
    pub allow_arguments: Vec<String>,
    pub version: KPEngineVersion,
    pub status: KPEngineStatus,

    // state
    pub groups: Vec<Vec<KPFilter>>,

    // context
    #[derivative(Debug = "ignore")]
    pub(crate) engine: Arc<Mutex<Engine>>,
    #[derivative(Debug = "ignore")]
    pub(crate) module: Arc<Mutex<Module>>,
    #[derivative(Debug = "ignore")]
    pub(crate) store: Arc<Mutex<Store<WasiP1Ctx>>>,
    #[derivative(Debug = "ignore")]
    pub(crate) linker: Arc<Mutex<Linker<WasiP1Ctx>>>,
    #[derivative(Debug = "ignore")]
    pub(crate) memory: Arc<Mutex<Memory>>,
    #[derivative(Debug = "ignore")]
    pub(crate) instance: Arc<Mutex<Instance>>,
}

impl KPEngine {
    pub async fn new(bytecode: Vec<u8>) -> Result<Self> {
        let engine = Engine::new(Config::new().async_support(true))?;

        let wasi_ctx = WasiCtxBuilder::new()
            .inherit_stdio()
            .build_p1();
        let mut store = Store::new(&engine, wasi_ctx);

        let mut linker: Linker<WasiP1Ctx> = Linker::new(&engine);
        wasmtime_wasi::preview1::add_to_linker_async(&mut linker, |t| t)?;

        let module = Module::from_binary(&engine, &bytecode)?;
        let instance = linker.instantiate_async(&mut store, &module).await?;
        let memory = instance.get_memory(&mut store, DEFAULT_MEMORY).ok_or_else(|| anyhow!("memory not found. memory: {}", DEFAULT_MEMORY))?;

        let mut engine = KPEngine {
            bytecode,
            app: "".to_string(),
            author: "".to_string(),
            media_type: Default::default(),
            sort_type: Default::default(),
            default_arguments: Default::default(),
            allow_arguments: vec![],
            version: Default::default(),
            status: Default::default(),
            groups: vec![],
            engine: Arc::new(Mutex::new(engine)),
            module: Arc::new(Mutex::new(module)),
            store: Arc::new(Mutex::new(store)),
            linker: Arc::new(Mutex::new(linker)),
            memory: Arc::new(Mutex::new(memory)),
            instance: Arc::new(Mutex::new(instance)),
        };

        // init
        engine.init().await?;
        engine.status = KPEngineStatus::Initialized;

        // set basic information
        engine.app = engine.get_app().await?;
        engine.author = engine.get_author().await?;
        engine.media_type = engine.get_media_type().await?;
        engine.version = KPEngineVersion::from(engine.get_version().await?)?;
        engine.sort_type = engine.get_sort_type().await?;

        let (groups, default_arguments, allow_arguments) = engine.get_groups().await?;
        engine.allow_arguments = allow_arguments;
        engine.default_arguments = default_arguments;
        engine.groups = groups;
        engine.status = KPEngineStatus::Loaded;

        Ok(engine)
    }
}

#[tokio::test]
async fn test_plugin() -> Result<()> {
    initialize();
    let wasm_path = env::var("TEXT_WASM_PATH")?;
    let file_data = fs::read(wasm_path)?;

    let engine = KPEngine::new(file_data).await?;
    info!("plugin: {:?}", engine);
    Ok(())
}

#[tokio::test]
async fn test_plugin_update_command() -> Result<()> {
    initialize();
    let wasm_path = env::var("TEXT_WASM_PATH")?;
    let file_data = fs::read(wasm_path)?;

    let engine = KPEngine::new(file_data).await?;
    info!("plugin: {:?}", engine);

    // get update command
    let mut update_arguments = BTreeMap::new();
    update_arguments.insert("text".to_string(), "changed".to_string());
    let command = engine.get_update_command(update_arguments).await?;
    info!("command: {:?}", command);
    Ok(())
}