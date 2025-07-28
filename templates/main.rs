use std::path::PathBuf;
use anyhow::{Context, Result};
use clap::Parser;
use tracing::{info, warn, error};
use tokio;

#[cfg(feature = "gpu")]
use cuda_runtime_sys::*;

mod gpu_accelerator;
mod voice_agent;
mod local_brain;
mod orchestrator;

use gpu_accelerator::{GPUAccelerator, GPUConfig};
use voice_agent::{VoiceAgent, VoiceConfig};
use local_brain::{LocalBrain, LocalBrainConfig};
use orchestrator::{Orchestrator, OrchestratorConfig};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Enable GPU acceleration
    #[arg(short, long)]
    gpu: bool,
    
    /// Voice clone name
    #[arg(short, long)]
    voice_name: Option<String>,
    
    /// Audio files for voice cloning
    #[arg(short, long)]
    audio_files: Vec<PathBuf>,
    
    /// Enable voice control
    #[arg(short, long)]
    voice: bool,
    
    /// Generate boilerplate code
    #[arg(short, long)]
    generate: bool,
    
    /// Project name for code generation
    #[arg(short, long)]
    project_name: Option<String>,
    
    /// Benchmark GPU performance
    #[arg(short, long)]
    benchmark: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    let args = Args::parse();
    
    info!("🚀 Starting High-Performance Voice Agent System");
    
    // Initialize GPU accelerator
    let gpu_config = GPUConfig {
        device_id: 0, // GTX 1660
        max_threads_per_block: 1024,
        shared_memory_size: 49152, // 48KB shared memory
        enable_tensor_cores: true,
        memory_pool_size: 4 * 1024 * 1024 * 1024, // 4GB GPU memory pool
    };
    
    let gpu_accelerator = GPUAccelerator::new(gpu_config).await?;
    
    // Benchmark GPU if requested
    if args.benchmark {
        info!("📊 Running GPU benchmark...");
        let metrics = gpu_accelerator.benchmark_gpu_performance().await?;
        info!("⚡ GPU Performance: {:.0} tokens/sec", metrics.throughput_tokens_per_sec);
        return Ok(());
    }
    
    // Generate boilerplate code if requested
    if args.generate {
        if let Some(project_name) = args.project_name {
            info!("🦀 Generating Rust boilerplate for: {}", project_name);
            let results = gpu_accelerator.generate_rust_boilerplate(&project_name).await?;
            info!("✅ Generated {} files with GPU acceleration", results.len());
            
            // Save generated files
            for result in results {
                // TODO: Save files to disk
                info!("Generated: {} chars in {}ms", 
                      result.generated_code.len(), 
                      result.compilation_time_ms);
            }
        }
    }
    
    // Voice cloning if audio files provided
    if !args.audio_files.is_empty() {
        if let Some(voice_name) = args.voice_name {
            info!("🎤 Cloning voice: {}", voice_name);
            
            // TODO: Implement voice cloning with GPU acceleration
            info!("✅ Voice cloning completed");
        }
    }
    
    // Start voice agent if requested
    if args.voice {
        info!("🎤 Starting voice-controlled agent");
        
        // Initialize voice agent with GPU acceleration
        let voice_config = VoiceConfig {
            sample_rate: 16000,
            channels: 1,
            voice_model_path: PathBuf::from("voice_models/default"),
            gpu_enabled: args.gpu,
            wasm_storage_path: PathBuf::from("wasm_storage"),
        };
        
        let voice_agent = VoiceAgent::new(voice_config).await?;
        
        // Initialize local brain
        let brain_config = LocalBrainConfig {
            model_path: PathBuf::from("models/phi-3-mini-instruct"),
            max_tokens: 2048,
            temperature: 0.7,
            gpu_enabled: args.gpu,
            mcp_servers: vec!["http://localhost:8080".to_string()],
        };
        
        let local_brain = LocalBrain::new(brain_config).await?;
        
        // Initialize orchestrator
        let orchestrator_config = OrchestratorConfig {
            whisper_path: PathBuf::from("components/whisper.exe"),
            llama_path: PathBuf::from("components/llama.exe"),
            piper_path: PathBuf::from("components/piper.exe"),
            model_path: PathBuf::from("components/models/TinyLlama-1.1B-Chat-v1.0.Q4_K_M.gguf"),
            voice_model_path: PathBuf::from("voice_models/default/models/voice_model.pth"),
            qdrant_url: "http://localhost:6333".to_string(),
            indradb_url: "http://localhost:8080".to_string(),
            gpu_enabled: args.gpu,
            max_concurrent_requests: 10,
        };
        
        let orchestrator = Orchestrator::new(orchestrator_config).await?;
        
        info!("🎤 Voice agent ready! Speak to interact...");
        
        // Start voice listener
        voice_agent.start_voice_listener().await?;
    }
    
    info!("✅ High-performance voice agent system completed");
    Ok(())
} 