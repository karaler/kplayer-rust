use crate::scene::engine::*;

// basic
impl KPEngine {
    async fn get_app(&self) -> Result<String> {
        let mut store_locker = self.store.lock().await;
        let mut store = store_locker.deref_mut();
        let instance = self.instance.lock().await;

        let func = instance.get_func(&mut store, "get_app")
            .ok_or_else(|| anyhow!("function not found"))?
            .typed::<(), (i64)>(&store)?;
        func.call_async(&mut store, ()).await?;

        return Err(anyhow!("invalid function extra found"));
    }
}

// memory
impl KPEngine {
    async fn allocate(&self, size: usize) -> Result<i32> {
        let mut store_locker = self.store.lock().await;
        let mut store = store_locker.deref_mut();
        let instance = self.instance.lock().await;

        let func = instance.get_typed_func::<(u32), i32>(&mut store, "allocate")?;
        let result = func.call_async(&mut store, size as u32).await?;
        Ok(result)
    }

    async fn deallocate(&self, ptr: i32, size: usize) -> Result<()> {
        let mut store_locker = self.store.lock().await;
        let mut store = store_locker.deref_mut();
        let instance = self.instance.lock().await;

        let func = instance.get_typed_func::<(i32, u32), ()>(&mut store, "deallocate")?;
        func.call_async(&mut store, (ptr, size as u32)).await?;
        Ok(())
    }

    async fn write_memory(&self, ptr: i32, bytes: &Vec<u8>) -> Result<()> {
        let mut store_locker = self.store.lock().await;
        let mut store = store_locker.deref_mut();
        let memory = self.memory.lock().await;
        memory.write(&mut store, ptr as usize, bytes.as_slice())?;
        Ok(())
    }

    async fn read_memory(&self, ptr: i32, size: usize) -> Result<Vec<u8>> {
        let mut store_locker = self.store.lock().await;
        let mut store = store_locker.deref_mut();
        let memory = self.memory.lock().await;

        let mut buffer = vec![0; size];
        memory.read(&mut store, ptr as usize, &mut buffer)?;
        Ok(buffer)
    }

    async fn read_memory_as_string(&self, ptr: i32, size: usize) -> Result<String> {
        let buf = self.read_memory(ptr, size).await?;
        let result = if let Ok(str) = std::str::from_utf8(&buf) {
            Ok(str.to_string())
        } else {
            Err(anyhow!("data is not valid UTF-8"))
        };

        // destroy memory
        self.deallocate(ptr, size).await?;
        result
    }

    async fn allocate_memory<F>(&self, bytes: &Vec<u8>, f: F) -> Result<i32> where
        F: Fn(Arc<Mutex<Store<WasiP1Ctx>>>, Arc<Mutex<Instance>>, i32, usize) -> Pin<Box<dyn Future<Output=Result<()>>>>,
    {
        let size = bytes.len();
        let ptr = self.allocate(size.clone()).await?;
        self.write_memory(ptr.clone(), bytes).await?;

        // call closure
        f(self.store.clone(), self.instance.clone(), ptr.clone(), size.clone()).await?;

        // destroy
        self.deallocate(ptr, size).await?;
        Ok(ptr)
    }
}

#[tokio::test]
async fn test_memory_write() -> Result<()> {
    initialize();
    let memory_wasm_path = env::var("MEMORY_WASM_PATH").unwrap();
    let file_data = fs::read(memory_wasm_path)?;

    let engine = KPEngine::new(file_data).await?;
    let input = "Hello, KPlayer!";
    engine.allocate_memory(&input.as_bytes().to_vec(), |store_arc, instance_arc, ptr, size| {
        Box::pin(async move {
            let mut store_locker = store_arc.lock().await;
            let mut store = store_locker.deref_mut();
            let instance = instance_arc.lock().await;

            let func = instance.get_typed_func::<(i32, u32), ()>(&mut store, "print_string")?;
            func.call_async(&mut store, (ptr, size as u32)).await?;
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
    let (ptr, size) = {
        let mut store_locker = engine.store.lock().await;
        let mut store = store_locker.deref_mut();
        let instance = engine.instance.lock().await;

        let func = instance.get_typed_func::<(), u64>(&mut store, "get_string")?;
        let memory_p = func.call_async(&mut store, ()).await?;
        memory_split!(memory_p)
    };

    let str = engine.read_memory_as_string(ptr, size as usize).await?;
    info!("Read string: {}", str);
    Ok(())
}