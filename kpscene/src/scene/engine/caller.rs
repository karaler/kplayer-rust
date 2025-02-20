use log::debug;
use crate::scene::engine::*;
use crate::scene::engine::vars::KPPluginInfo;
use crate::scene::scene::KPSceneSortType;

// lifecycle
impl KPEngine {
    pub(crate) async fn init(&self) -> Result<()> {
        let mut store_locker = self.store.lock().await;
        let mut store = store_locker.deref_mut();
        let instance = self.instance.lock().await;

        let func = instance.get_func(&mut store, "init")
            .ok_or_else(|| anyhow!("function not found"))?
            .typed::<(), (i32)>(&store)?;
        let result = func.call_async(&mut store, ()).await?;
        if result < 0 {
            return Err(anyhow!("initialization function returned an error"));
        }
        Ok(())
    }
}

// information
impl KPEngine {
    pub(crate) async fn get_app(&self) -> Result<String> {
        assert_eq!(self.status, KPEngineStatus::Initialized);

        let memory_p = {
            let mut store_locker = self.store.lock().await;
            let mut store = store_locker.deref_mut();
            let instance = self.instance.lock().await;

            let func = instance.get_func(&mut store, "get_app")
                .ok_or_else(|| anyhow!("function not found"))?
                .typed::<(), (MemoryPoint)>(&store)?;
            func.call_async(&mut store, ()).await?
        };

        // get string
        let app = self.read_memory_as_string(memory_p).await?;
        Ok(app)
    }

    pub(crate) async fn get_author(&self) -> Result<String> {
        assert_eq!(self.status, KPEngineStatus::Initialized);
        let memory_p = {
            let mut store_locker = self.store.lock().await;
            let mut store = store_locker.deref_mut();
            let instance = self.instance.lock().await;

            let func = instance.get_func(&mut store, "get_author")
                .ok_or_else(|| anyhow!("function not found"))?
                .typed::<(), (MemoryPoint)>(&store)?;
            func.call_async(&mut store, ()).await?
        };

        // get string
        let author = self.read_memory_as_string(memory_p).await?;
        Ok(author)
    }

    pub(crate) async fn get_media_type(&self) -> Result<KPAVMediaType> {
        assert_eq!(self.status, KPEngineStatus::Initialized);
        let mut store_locker = self.store.lock().await;
        let mut store = store_locker.deref_mut();
        let instance = self.instance.lock().await;

        let func = instance.get_func(&mut store, "get_media_type")
            .ok_or_else(|| anyhow!("function not found"))?
            .typed::<(), (i32)>(&store)?;
        let media_type = func.call_async(&mut store, ()).await?;
        Ok(KPAVMediaType::from_i32(media_type))
    }

    pub(crate) async fn get_sort_type(&self) -> Result<KPSceneSortType> {
        assert_eq!(self.status, KPEngineStatus::Initialized);
        let mut store_locker = self.store.lock().await;
        let mut store = store_locker.deref_mut();
        let instance = self.instance.lock().await;

        let func = instance.get_func(&mut store, "get_sort_type")
            .ok_or_else(|| anyhow!("function not found"))?
            .typed::<(), (i32)>(&store)?;
        let sort_type = func.call_async(&mut store, ()).await?;
        Ok(KPSceneSortType::from_i32(sort_type))
    }


    pub(crate) async fn get_version(&self) -> Result<String> {
        assert_eq!(self.status, KPEngineStatus::Initialized);
        let memory_p = {
            let mut store_locker = self.store.lock().await;
            let mut store = store_locker.deref_mut();
            let instance = self.instance.lock().await;

            let func = instance.get_func(&mut store, "get_version")
                .ok_or_else(|| anyhow!("function not found"))?
                .typed::<(), (MemoryPoint)>(&store)?;
            func.call_async(&mut store, ()).await?
        };

        // get string
        let version = self.read_memory_as_string(memory_p).await?;
        Ok(version)
    }


    pub(crate) async fn get_update_command(&self, arguments: BTreeMap<String, String>) -> Result<BTreeMap<String, BTreeMap<String, String>>> {
        assert_eq!(self.status, KPEngineStatus::Loaded);
        // serialize `arguments` to a JSON string
        let arguments_json = serde_json::to_string(&arguments)?;

        // write the JSON string to memory
        let memory_point = self.allocate(arguments_json.len()).await?;
        self.write_memory(&memory_point, &arguments_json.into_bytes()).await?;

        let memory_p = {
            let mut store_locker = self.store.lock().await;
            let mut store = store_locker.deref_mut();
            let instance = self.instance.lock().await;

            let func = instance.get_func(&mut store, "get_update_command")
                .ok_or_else(|| anyhow!("function not found"))?
                .typed::<(MemoryPoint), (MemoryPoint)>(&store)?;
            func.call_async(&mut store, (memory_point)).await?
        };

        // get string
        let update_command = self.read_memory_as_string(memory_p).await?;

        // deserialize the JSON string to a BtreeMap<String, BTreeMap<String, String>>
        let update_command: BTreeMap<String, BTreeMap<String, String>> = serde_json::from_str(&update_command)?;
        debug!("update_command: {:?}", update_command);
        Ok(update_command)
    }

    pub(crate) async fn get_groups(&self) -> Result<(Vec<Vec<KPFilter>>, BTreeMap<String, String>, Vec<String>)> {
        assert_eq!(self.status, KPEngineStatus::Initialized);
        let memory_p = {
            let mut store_locker = self.store.lock().await;
            let mut store = store_locker.deref_mut();
            let instance = self.instance.lock().await;

            let func = instance.get_func(&mut store, "get_groups")
                .ok_or_else(|| anyhow!("function not found"))?
                .typed::<(), (MemoryPoint)>(&store)?;
            func.call_async(&mut store, ()).await?
        };

        // get string
        let groups_str = self.read_memory_as_string(memory_p).await?;
        let groups: Vec<Vec<KPPluginInfo>> = serde_json::from_str(&groups_str)?;

        let mut items: Vec<Vec<KPFilter>> = Vec::new();
        let mut default_arguments = BTreeMap::new();
        let mut allow_arguments = Vec::new();

        for group in groups {
            let mut filter_item = Vec::new();
            for group_item in group {
                // replace custom argument
                let group_allow_arguments = group_item.allow_arguments.clone();
                let mut group_combine_arguments = group_item.default_arguments.clone();
                for (key, value) in self.custom_arguments.iter() {
                    if group_allow_arguments.contains(key) {
                        group_combine_arguments.insert(key.clone(), value.clone());
                    }
                }
                debug!("plugin group. name: {}, allow_arguments: {:?}, default_arguments: {:?}, combine_arguments: {:?}",
                    group_item.name, allow_arguments, group_allow_arguments, group_combine_arguments);

                // create filter
                let filter = KPFilter::new(group_item.name, group_item.filter_name, group_combine_arguments, group_item.allow_arguments.clone())?;
                filter_item.push(filter);

                // default_arguments
                default_arguments.extend(group_item.default_arguments);
                allow_arguments.extend(group_item.allow_arguments);
            }
            items.push(filter_item);
        }

        Ok((items, default_arguments, allow_arguments))
    }
}

// memory
impl KPEngine {
    async fn allocate(&self, size: usize) -> Result<MemoryPoint> {
        let mut store_locker = self.store.lock().await;
        let mut store = store_locker.deref_mut();
        let instance = self.instance.lock().await;

        let func = instance.get_typed_func::<(u32), MemoryPoint>(&mut store, "allocate")?;
        let result = func.call_async(&mut store, size as u32).await?;
        Ok(result)
    }

    async fn deallocate(&self, memory_point: MemoryPoint) -> Result<()> {
        let mut store_locker = self.store.lock().await;
        let mut store = store_locker.deref_mut();
        let instance = self.instance.lock().await;

        let func = instance.get_typed_func::<(MemoryPoint), ()>(&mut store, "deallocate")?;
        func.call_async(&mut store, memory_point).await?;
        Ok(())
    }

    async fn write_memory(&self, memory_point: &MemoryPoint, bytes: &Vec<u8>) -> Result<()> {
        let mut store_locker = self.store.lock().await;
        let mut store = store_locker.deref_mut();
        let memory = self.memory.lock().await;

        let (ptr, _) = memory_split!(memory_point);
        memory.write(&mut store, ptr as usize, bytes.as_slice())?;
        Ok(())
    }

    async fn read_memory(&self, memory_point: &MemoryPoint) -> Result<Vec<u8>> {
        let mut store_locker = self.store.lock().await;
        let mut store = store_locker.deref_mut();
        let memory = self.memory.lock().await;

        let (ptr, size) = memory_split!(memory_point);
        let mut buffer = vec![0; size];
        memory.read(&mut store, ptr as usize, &mut buffer)?;
        Ok(buffer)
    }

    async fn read_memory_as_string(&self, memory_point: MemoryPoint) -> Result<String> {
        let buf = self.read_memory(&memory_point).await?;
        let result = if let Ok(str) = std::str::from_utf8(&buf) {
            Ok(str.to_string())
        } else {
            Err(anyhow!("data is not valid UTF-8"))
        };

        // destroy memory
        self.deallocate(memory_point).await?;
        result
    }

    async fn allocate_memory<F>(&self, bytes: &Vec<u8>, f: F) -> Result<()>
    where
        F: Fn(Arc<Mutex<Store<WasiP1Ctx>>>, Arc<Mutex<Instance>>, MemoryPoint) -> Pin<Box<dyn Future<Output=Result<()>>>>,
    {
        let memory_p = self.allocate(bytes.len()).await?;
        self.write_memory(&memory_p, bytes).await?;

        // call closure
        f(self.store.clone(), self.instance.clone(), memory_p).await?;

        // destroy
        self.deallocate(memory_p).await?;
        Ok(())
    }
}

#[tokio::test]
async fn test_memory_write() -> Result<()> {
    initialize();
    let memory_wasm_path = env::var("MEMORY_WASM_PATH").unwrap();
    let file_data = fs::read(memory_wasm_path)?;

    let engine = KPEngine::new(file_data).await?;
    let input = "Hello, KPlayer!";
    engine.allocate_memory(&input.as_bytes().to_vec(), |store_arc, instance_arc, memory_p| {
        Box::pin(async move {
            let mut store_locker = store_arc.lock().await;
            let mut store = store_locker.deref_mut();
            let instance = instance_arc.lock().await;

            let func = instance.get_typed_func::<(MemoryPoint), ()>(&mut store, "print_string")?;
            func.call_async(&mut store, memory_p).await?;
            Ok(())
        })
    }).await?;
    Ok(())
}

#[tokio::test]
async fn test_memory_read() -> Result<()> {
    initialize();
    let memory_wasm_path = env::var("MEMORY_WASM_PATH").unwrap();
    let file_data = fs::read(memory_wasm_path)?;

    let engine = KPEngine::new(file_data).await?;
    let memory_p = {
        let mut store_locker = engine.store.lock().await;
        let mut store = store_locker.deref_mut();
        let instance = engine.instance.lock().await;

        let func = instance.get_typed_func::<(), u64>(&mut store, "get_string")?;
        func.call_async(&mut store, ()).await?
    };

    let str = engine.read_memory_as_string(memory_p).await?;
    info!("Read string: {}", str);
    Ok(())
}


#[tokio::test]
async fn test_plugin() -> Result<()> {
    initialize();
    let wasm_path = env::var("TEXT_WASM_PATH").unwrap();
    let file_data = fs::read(wasm_path)?;

    let engine = KPEngine::new(file_data).await?;
    let app = engine.get_app().await?;
    info!("app name: {}",app);
    Ok(())
}