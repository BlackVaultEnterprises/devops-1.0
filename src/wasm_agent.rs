use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use wasmtime::{Engine, Instance, Module, Store};
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder};
use tokio::fs;
use tracing::{info, warn, error};

#[derive(Debug, Serialize, Deserialize)]
pub struct WasmAnalysis {
    pub compile_time: f64,
    pub binary_size: usize,
    pub optimization_suggestions: Vec<String>,
    pub performance_score: f32,
    pub wasm_compatibility: bool,
    pub memory_usage: usize,
    pub export_functions: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WasmOptimization {
    pub name: String,
    pub description: String,
    pub impact: String,
    pub code_example: String,
}

pub struct WasmAgent {
    engine: Engine,
    store: Store<WasiCtx>,
    optimizations: HashMap<String, WasmOptimization>,
}

impl WasmAgent {
    pub async fn new() -> Result<Self> {
        info!("Initializing WASM Agent...");
        
        let engine = Engine::default();
        let wasi = WasiCtxBuilder::new()
            .inherit_stdio()
            .inherit_args()?
            .build();
        let store = Store::new(&engine, wasi);
        
        let mut optimizations = HashMap::new();
        
        // Add common WASM optimizations
        optimizations.insert(
            "no_std".to_string(),
            WasmOptimization {
                name: "Use no_std".to_string(),
                description: "Remove standard library dependencies for smaller WASM size".to_string(),
                impact: "High".to_string(),
                code_example: "#![no_std]\nuse core::prelude::*;".to_string(),
            },
        );
        
        optimizations.insert(
            "panic_abort".to_string(),
            WasmOptimization {
                name: "Use panic_abort".to_string(),
                description: "Abort on panic instead of unwinding for smaller size".to_string(),
                impact: "Medium".to_string(),
                code_example: "[profile.release]\npanic = \"abort\"".to_string(),
            },
        );
        
        optimizations.insert(
            "lto".to_string(),
            WasmOptimization {
                name: "Enable LTO".to_string(),
                description: "Link Time Optimization for better performance".to_string(),
                impact: "High".to_string(),
                code_example: "[profile.release]\nlto = true".to_string(),
            },
        );
        
        Ok(Self {
            engine,
            store,
            optimizations,
        })
    }
    
    pub async fn analyze_rust_file(&self, content: &str) -> Result<WasmAnalysis> {
        info!("Analyzing Rust file for WASM compatibility...");
        
        let start_time = std::time::Instant::now();
        
        // Check for WASM compatibility issues
        let mut suggestions = Vec::new();
        let mut compatibility_score = 1.0;
        
        // Check for std usage
        if content.contains("use std::") && !content.contains("#![no_std]") {
            suggestions.push("Consider using no_std for smaller WASM size".to_string());
            compatibility_score -= 0.2;
        }
        
        // Check for file system operations
        if content.contains("std::fs::") || content.contains("File::") {
            suggestions.push("File system operations are not available in WASM".to_string());
            compatibility_score -= 0.3;
        }
        
        // Check for threading
        if content.contains("std::thread::") || content.contains("spawn") {
            suggestions.push("Threading is not available in WASM".to_string());
            compatibility_score -= 0.4;
        }
        
        // Check for network operations
        if content.contains("TcpStream") || content.contains("UdpSocket") {
            suggestions.push("Network operations require WASI or web APIs".to_string());
            compatibility_score -= 0.2;
        }
        
        // Check for dynamic allocations
        if content.contains("Box::new") || content.contains("Vec::new") {
            suggestions.push("Consider using static allocations where possible".to_string());
        }
        
        // Check for panic handling
        if content.contains("unwrap()") || content.contains("expect(") {
            suggestions.push("Consider using Result types instead of panicking".to_string());
            compatibility_score -= 0.1;
        }
        
        // Estimate binary size based on code complexity
        let lines = content.lines().count();
        let estimated_size = lines * 100; // Rough estimate
        
        let compile_time = start_time.elapsed().as_secs_f64();
        
        Ok(WasmAnalysis {
            compile_time,
            binary_size: estimated_size,
            optimization_suggestions: suggestions,
            performance_score: compatibility_score,
            wasm_compatibility: compatibility_score > 0.5,
            memory_usage: estimated_size / 2,
            export_functions: self.extract_export_functions(content),
        })
    }
    
    fn extract_export_functions(&self, content: &str) -> Vec<String> {
        let mut functions = Vec::new();
        
        for line in content.lines() {
            if line.contains("#[no_mangle]") || line.contains("pub extern") {
                // Extract function name
                if let Some(name) = line.split("fn ").nth(1) {
                    if let Some(func_name) = name.split('(').next() {
                        functions.push(func_name.trim().to_string());
                    }
                }
            }
        }
        
        functions
    }
    
    pub async fn compile_to_wasm(&self, rust_file: &Path) -> Result<Vec<u8>> {
        info!("Compiling Rust file to WASM: {}", rust_file.display());
        
        // Create temporary directory for compilation
        let temp_dir = std::env::temp_dir().join("wasm_compile");
        fs::create_dir_all(&temp_dir).await?;
        
        // Copy file to temp directory
        let temp_file = temp_dir.join("main.rs");
        fs::copy(rust_file, &temp_file).await?;
        
        // Create Cargo.toml for WASM compilation
        let cargo_toml = format!(
            r#"[package]
name = "wasm_module"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
wasm-bindgen = "0.2"

[profile.release]
opt-level = 3
lto = true
codegen-units = 1
panic = "abort"
"#
        );
        
        let cargo_file = temp_dir.join("Cargo.toml");
        fs::write(&cargo_file, cargo_toml).await?;
        
        // Run wasm-pack build
        let output = tokio::process::Command::new("wasm-pack")
            .args(["build", "--target", "web", "--release"])
            .current_dir(&temp_dir)
            .output()
            .await
            .context("Failed to run wasm-pack")?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error!("WASM compilation failed: {}", stderr);
            return Err(anyhow::anyhow!("WASM compilation failed"));
        }
        
        // Read the generated WASM file
        let wasm_file = temp_dir.join("pkg").join("wasm_module_bg.wasm");
        let wasm_bytes = fs::read(&wasm_file).await?;
        
        info!("WASM compilation successful, size: {} bytes", wasm_bytes.len());
        
        Ok(wasm_bytes)
    }
    
    pub async fn analyze_wasm_module(&self, wasm_bytes: &[u8]) -> Result<WasmAnalysis> {
        info!("Analyzing WASM module...");
        
        let module = Module::new(&self.engine, wasm_bytes)?;
        
        // Analyze exports
        let mut export_functions = Vec::new();
        for export in module.exports() {
            if let wasmtime::ExternType::Func(_) = export.ty() {
                export_functions.push(export.name().to_string());
            }
        }
        
        // Estimate performance based on module size and complexity
        let binary_size = wasm_bytes.len();
        let performance_score = if binary_size < 100_000 {
            0.9
        } else if binary_size < 500_000 {
            0.7
        } else {
            0.5
        };
        
        let mut suggestions = Vec::new();
        
        if binary_size > 1_000_000 {
            suggestions.push("WASM module is very large, consider optimizations".to_string());
        }
        
        if export_functions.is_empty() {
            suggestions.push("No exported functions found".to_string());
        }
        
        Ok(WasmAnalysis {
            compile_time: 0.0, // Not applicable for pre-compiled WASM
            binary_size,
            optimization_suggestions: suggestions,
            performance_score,
            wasm_compatibility: true,
            memory_usage: binary_size / 2,
            export_functions,
        })
    }
    
    pub async fn optimize_wasm(&self, wasm_bytes: &[u8]) -> Result<Vec<u8>> {
        info!("Optimizing WASM module...");
        
        // Use wasm-opt if available
        let temp_file = std::env::temp_dir().join("input.wasm");
        fs::write(&temp_file, wasm_bytes).await?;
        
        let output_file = std::env::temp_dir().join("optimized.wasm");
        
        let output = tokio::process::Command::new("wasm-opt")
            .args(["-O4", "-o", output_file.to_str().unwrap(), temp_file.to_str().unwrap()])
            .output()
            .await;
        
        match output {
            Ok(result) if result.status.success() => {
                let optimized_bytes = fs::read(&output_file).await?;
                info!("WASM optimization successful, size reduced from {} to {} bytes", 
                      wasm_bytes.len(), optimized_bytes.len());
                Ok(optimized_bytes)
            }
            _ => {
                warn!("wasm-opt not available, returning original WASM");
                Ok(wasm_bytes.to_vec())
            }
        }
    }
    
    pub fn get_optimization_suggestions(&self, analysis: &WasmAnalysis) -> Vec<WasmOptimization> {
        let mut suggestions = Vec::new();
        
        if analysis.binary_size > 500_000 {
            suggestions.push(self.optimizations.get("no_std").unwrap().clone());
        }
        
        if analysis.performance_score < 0.8 {
            suggestions.push(self.optimizations.get("lto").unwrap().clone());
        }
        
        if analysis.performance_score < 0.7 {
            suggestions.push(self.optimizations.get("panic_abort").unwrap().clone());
        }
        
        suggestions
    }
    
    pub async fn create_wasm_bindings(&self, rust_content: &str) -> Result<String> {
        info!("Generating WASM bindings...");
        
        let mut bindings = String::new();
        bindings.push_str("use wasm_bindgen::prelude::*;\n\n");
        
        // Extract public functions and create bindings
        for line in rust_content.lines() {
            if line.contains("pub fn ") {
                let binding = self.create_function_binding(line);
                bindings.push_str(&binding);
                bindings.push('\n');
            }
        }
        
        Ok(bindings)
    }
    
    fn create_function_binding(&self, function_line: &str) -> String {
        // Simple binding generation
        if let Some(func_name) = function_line.split("fn ").nth(1) {
            if let Some(name) = func_name.split('(').next() {
                return format!("#[wasm_bindgen]\npub fn {}() {{\n    // WASM binding\n}}\n", name.trim());
            }
        }
        
        String::new()
    }
} 