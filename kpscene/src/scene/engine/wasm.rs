use std::sync::Arc;
use wasmtime::{Config, Engine, Memory};
use wasmtime::Module;
use wasmtime_wasi::preview1::WasiP1Ctx;
use wasmtime_wasi::WasiCtxBuilder;
use crate::scene::engine::*;

const DEFAULT_MODULE: &str = "";
const DEFAULT_MEMORY: &str = "memory";

pub struct KPEngine {
    pub(crate) bytecode: Vec<u8>,
    pub(crate) app: String,
    pub(crate) author: String,
    pub(crate) media_type: KPAVMediaType,
    pub(crate) arguments: HashMap<String, String>,
    pub(crate) version: KPEngineVersion,
    pub(crate) status: KPEngineStatus,

    // state
    pub(crate) groups: Vec<Vec<KPFilter>>,

    // context
    pub(crate) engine: Arc<Mutex<Engine>>,
    pub(crate) module: Arc<Mutex<Module>>,
    pub(crate) store: Arc<Mutex<Store<WasiP1Ctx>>>,
    pub(crate) linker: Arc<Mutex<Linker<WasiP1Ctx>>>,
    pub(crate) memory: Arc<Mutex<Memory>>,
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
            arguments: Default::default(),
            version: KPEngineVersion {},
            status: KPEngineStatus::None,
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
        let app = engine.get_app().await?;
        engine.app = app;

        Ok(engine)
    }
}
