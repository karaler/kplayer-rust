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

        let func = instance.get_typed_func::<(u64), i32>(&mut store, "allocate")?;
        let result = func.call_async(&mut store, size as u64).await?;
        Ok(result)
    }

    async fn deallocate(&self, ptr: i32, size: usize) -> Result<()> {
        let mut store_locker = self.store.lock().await;
        let mut store = store_locker.deref_mut();
        let instance = self.instance.lock().await;

        let func = instance.get_typed_func::<(i32, u64), ()>(&mut store, "deallocate")?;
        func.call_async(&mut store, (ptr, size as u64)).await?;
        Ok(())
    }

    async fn write_memory(&self, ptr: i32, bytes: &Vec<u8>) -> Result<()> {
        let mut store_locker = self.store.lock().await;
        let mut store = store_locker.deref_mut();
        let memory = self.memory.lock().await;
        memory.write(&mut store, ptr as usize, bytes.as_slice())?;
        Ok(())
    }

    async fn allocate_memory(&self, bytes: &Vec<u8>) -> Result<i32> {
        let size = bytes.len();
        let ptr = self.allocate(size).await?;
        self.write_memory(ptr.clone(), bytes).await?;
        Ok(ptr)
    }
}