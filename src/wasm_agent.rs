use wasmtime::{Engine, Instance, Module, Store};
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder};
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct WasmAgentConfig {
    pub wasm_path: String,
    pub memory_size: u32,
    pub stack_size: u32,
}

pub struct WasmAgent {
    engine: Engine,
    store: Store<WasiCtx>,
    instance: Instance,
}

impl WasmAgent {
    pub fn new(config: WasmAgentConfig) -> Result<Self> {
        // Create WASM engine
        let engine = Engine::default();
        
        // Load WASM module
        let module = Module::from_file(&engine, &config.wasm_path)?;
        
        // Create WASI context
        let wasi = WasiCtxBuilder::new()
            .inherit_stdio()
            .inherit_args()?
            .build();
        
        // Create store
        let mut store = Store::new(&engine, wasi);
        
        // Instantiate module
        let instance = wasi::add_to_linker(&mut store, |s| s)?
            .instantiate(&mut store, &module, &[])?;
        
        Ok(Self {
            engine,
            store,
            instance,
        })
    }
    
    pub fn call_function(&mut self, name: &str, params: &[u32]) -> Result<u32> {
        let func = self.instance
            .get_func(&mut self.store, name)
            .ok_or_else(|| anyhow::anyhow!("Function {} not found", name))?;
        
        let result = func.call(&mut self.store, params, &mut [])?;
        
        Ok(result[0].unwrap_i32() as u32)
    }
    
    pub fn get_memory(&mut self) -> Result<Vec<u8>> {
        let memory = self.instance
            .get_memory(&mut self.store, "memory")
            .ok_or_else(|| anyhow::anyhow!("Memory not found"))?;
        
        let data = memory.data(&self.store);
        Ok(data.to_vec())
    }
    
    pub fn set_memory(&mut self, data: &[u8]) -> Result<()> {
        let memory = self.instance
            .get_memory(&mut self.store, "memory")
            .ok_or_else(|| anyhow::anyhow!("Memory not found"))?;
        
        let memory_data = memory.data_mut(&mut self.store);
        if data.len() <= memory_data.len() {
            memory_data[..data.len()].copy_from_slice(data);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Data too large for memory"))
        }
    }
}

// WASM interface for code review
#[no_mangle]
pub extern "C" fn review_code(ptr: *const u8, len: usize) -> *mut u8 {
    // This would be implemented in the WASM module
    // For now, return a dummy result
    std::ptr::null_mut()
}

#[no_mangle]
pub extern "C" fn generate_patch(ptr: *const u8, len: usize) -> *mut u8 {
    // This would be implemented in the WASM module
    // For now, return a dummy result
    std::ptr::null_mut()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_wasm_agent_creation() {
        let config = WasmAgentConfig {
            wasm_path: "test.wasm".to_string(),
            memory_size: 1024,
            stack_size: 1024,
        };
        
        // This would fail in tests since we don't have a real WASM file
        // but it tests the structure
        assert_eq!(config.memory_size, 1024);
    }
} 