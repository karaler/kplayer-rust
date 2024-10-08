use std::sync::Arc;
use wasmtime::{Config, Engine, Memory, MemoryType};
use wasmtime::component::__internal::wasmtime_environ::wasmparser::types::ModuleType;
use wasmtime::Module;
use wasmtime_wasi::preview1::WasiP1Ctx;
use wasmtime_wasi::WasiCtxBuilder;
use crate::scene::engine::*;

const DEFAULT_MODULE: &str = "";
const DEFAULT_MEMORY: &str = "memory";

pub struct KPEngine {
    bytecode: Vec<u8>,
    app: String,
    author: String,
    media_type: KPAVMediaType,
    arguments: HashMap<String, String>,
    version: KPEngineVersion,
    status: KPEngineStatus,

    // state
    groups: Vec<Vec<KPFilter>>,

    // context
    pub(crate) engine: Arc<Mutex<Engine>>,
    pub(crate) module: Arc<Mutex<Module>>,
    pub(crate) store: Arc<Mutex<Store<WasiP1Ctx>>>,
    pub(crate) linker: Arc<Mutex<Linker<WasiP1Ctx>>>,
    pub(crate) memory: Arc<Mutex<Memory>>,
    pub(crate) instance: Arc<Mutex<Instance>>,
}

impl KPEngine {
    fn init(&mut self, bytecode: &Vec<u8>) -> Result<()> {
        let mut config = Config::new();
        config.async_support(true);

        // memory
        let memory_ty = MemoryType::new(1, None);

        let engine = Engine::new(&config)?;
        let mut linker: Linker<WasiP1Ctx> = Linker::new(&engine);
        wasmtime_wasi::preview1::add_to_linker_async(&mut linker, |t| t)?;

        let module = Module::from_binary(&engine, bytecode)?;
        let wasi_ctx = WasiCtxBuilder::new().inherit_stdio().build_p1();
        let mut store = Store::new(&engine, wasi_ctx);
        let instance = linker.instantiate(&mut store, &module)?;
        let memory = instance.get_memory(&mut store, DEFAULT_MEMORY).ok_or_else(|| anyhow!("memory not found. memory: {}", DEFAULT_MEMORY))?;

        self.linker = Arc::new(Mutex::new(linker));
        self.module = Arc::new(Mutex::new(module));
        self.store = Arc::new(Mutex::new(store));
        self.engine = Arc::new(Mutex::new(engine));
        self.instance = Arc::new(Mutex::new(instance));
        self.memory = Arc::new(Mutex::new(memory));
        Ok(())
    }
}