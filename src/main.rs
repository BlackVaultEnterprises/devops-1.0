use std::path::PathBuf;
use std::process::Command;
use anyhow::{Context, Result};
use clap::Parser;
use serde::{Deserialize, Serialize};
use tokio::fs;
use tracing::{info, warn, error};
use walkdir::WalkDir;
use wasmtime::{Engine, Instance, Module, Store};
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder};
use axum::{
    routing::{get, post},
    http::StatusCode,
    Json, Router,
};
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};

mod wasm_agent;
mod llm_agent;
mod memory_system;
mod code_analyzer;
mod voice_agent;
mod local_brain;
mod orchestrator;
mod gpu_accelerator;

use wasm_agent::WasmAgent;
use llm_agent::LlmAgent;
use memory_system::MemorySystem;
use code_analyzer::CodeAnalyzer;
use voice_agent::{VoiceAgent, VoiceConfig};
use local_brain::{LocalBrain, LocalBrainConfig};
use orchestrator::{Orchestrator, OrchestratorConfig};
use gpu_accelerator::{GPUAccelerator, GPUConfig};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the codebase to review
    #[arg(short, long, default_value = "./src")]
    path: PathBuf,
    
    /// Output file for review results
    #[arg(short, long)]
    output: Option<PathBuf>,
    
    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,
    
    /// Run in interactive mode
    #[arg(short, long)]
    interactive: bool,
    
    /// Start web server for WASM hosting
    #[arg(short, long)]
    web: bool,
    
    /// Port for web server
    #[arg(short, long, default_value = "8080")]
    port: u16,
    
    /// Enable voice control
    #[arg(short, long)]
    voice: bool,
    
    /// Voice clone audio files
    #[arg(short, long)]
    voice_files: Vec<PathBuf>,
    
    /// Voice clone name
    #[arg(short, long)]
    voice_name: Option<String>,
    
    /// Enable GPU acceleration
    #[arg(short, long)]
    gpu: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct CodeReview {
    id: String,
    file_path: String,
    issues: Vec<Issue>,
    suggestions: Vec<Suggestion>,
    score: f32,
    timestamp: DateTime<Utc>,
    wasm_analysis: Option<WasmAnalysis>,
    llm_analysis: Option<LlmAnalysis>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Issue {
    severity: Severity,
    message: String,
    line: Option<usize>,
    code: Option<String>,
    wasm_context: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Suggestion {
    title: String,
    description: String,
    code: Option<String>,
    impact: Impact,
    wasm_optimization: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct WasmAnalysis {
    compile_time: f64,
    binary_size: usize,
    optimization_suggestions: Vec<String>,
    performance_score: f32,
}

#[derive(Debug, Serialize, Deserialize)]
struct LlmAnalysis {
    complexity_score: f32,
    maintainability_score: f32,
    security_score: f32,
    ai_suggestions: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Serialize, Deserialize)]
enum Impact {
    Low,
    Medium,
    High,
}

struct DevAgent {
    args: Args,
    wasm_agent: WasmAgent,
    llm_agent: LlmAgent,
    memory_system: MemorySystem,
    code_analyzer: CodeAnalyzer,
    voice_agent: Option<VoiceAgent>,
    local_brain: Option<LocalBrain>,
    orchestrator: Option<Orchestrator>,
}

impl DevAgent {
    async fn new(args: Args) -> Result<Self> {
        info!("Initializing DevAgent with WASM and LLM support...");
        
        let wasm_agent = WasmAgent::new().await?;
        let llm_agent = LlmAgent::new().await?;
        let memory_system = MemorySystem::new().await?;
        let code_analyzer = CodeAnalyzer::new().await?;
        
        Ok(Self {
            args,
            wasm_agent,
            llm_agent,
            memory_system,
            code_analyzer,
        })
    }
    
    async fn review_codebase(&self) -> Result<Vec<CodeReview>> {
        info!("Starting comprehensive codebase review with WASM and LLM analysis");
        
        let mut reviews = Vec::new();
        
        // Walk through the codebase
        for entry in WalkDir::new(&self.args.path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let file_path = entry.path();
            
            if !self.is_code_file(file_path) {
                continue;
            }
            
            info!("Reviewing file: {}", file_path.display());
            
            match self.review_file(file_path).await {
                Ok(review) => reviews.push(review),
                Err(e) => {
                    error!("Failed to review {}: {}", file_path.display(), e);
                }
            }
        }
        
        info!("Completed codebase review. Found {} files to review.", reviews.len());
        Ok(reviews)
    }
    
    fn is_code_file(&self, path: &std::path::Path) -> bool {
        let extensions = ["rs", "js", "ts", "py", "java", "cpp", "c", "go", "php", "wasm"];
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| extensions.contains(&ext))
            .unwrap_or(false)
    }
    
    async fn review_file(&self, file_path: &std::path::Path) -> Result<CodeReview> {
        let content = fs::read_to_string(file_path).await
            .context("Failed to read file")?;
        
        let file_id = Uuid::new_v4().to_string();
        
        // Store in memory system
        self.memory_system.store_file(&file_id, &content).await?;
        
        // Static analysis
        let issues = self.code_analyzer.analyze_code(&content, file_path).await?;
        let suggestions = self.code_analyzer.generate_suggestions(&content, file_path).await?;
        let score = self.code_analyzer.calculate_score(&content);
        
        // WASM analysis for Rust files
        let wasm_analysis = if file_path.extension().map_or(false, |ext| ext == "rs") {
            Some(self.wasm_agent.analyze_rust_file(&content).await?)
        } else {
            None
        };
        
        // LLM analysis
        let llm_analysis = Some(self.llm_agent.analyze_code(&content, file_path).await?);
        
        Ok(CodeReview {
            id: file_id,
            file_path: file_path.to_string_lossy().to_string(),
            issues,
            suggestions,
            score,
            timestamp: Utc::now(),
            wasm_analysis,
            llm_analysis,
        })
    }
    
    async fn save_reviews(&self, reviews: &[CodeReview]) -> Result<()> {
        let output_path = self.args.output.clone()
            .unwrap_or_else(|| PathBuf::from("code_review_results.json"));
        
        let json = serde_json::to_string_pretty(reviews)
            .context("Failed to serialize reviews")?;
        
        fs::write(&output_path, json).await
            .context("Failed to write review results")?;
        
        info!("Review results saved to: {}", output_path.display());
        Ok(())
    }
    
    async fn generate_patches(&self, reviews: &[CodeReview]) -> Result<()> {
        info!("Generating patches with WASM optimizations...");
        
        for review in reviews {
            for suggestion in &review.suggestions {
                if let Some(code) = &suggestion.code {
                    let patch_name = format!("{}_{}.patch", 
                        review.file_path.replace('/', "_").replace('\\', "_"),
                        suggestion.title.replace(' ', "_")
                    );
                    
                    let patch_content = format!(
                        "--- {}\n+++ {}\n@@ -1,1 +1,1 @@\n{}\n",
                        review.file_path, review.file_path, code
                    );
                    
                    fs::write(&patch_name, patch_content).await
                        .context("Failed to write patch file")?;
                    
                    info!("Generated patch: {}", patch_name);
                }
            }
        }
        
        Ok(())
    }
    
    async fn commit_changes(&self) -> Result<()> {
        info!("Committing changes to git...");
        
        let status = Command::new("git")
            .args(["add", "."])
            .status()
            .context("Failed to git add")?;
        
        if !status.success() {
            warn!("Git add failed");
            return Ok(());
        }
        
        let status = Command::new("git")
            .args(["commit", "-m", "Auto-generated code improvements from DevAgent with WASM optimizations"])
            .status()
            .context("Failed to git commit")?;
        
        if status.success() {
            info!("Changes committed successfully");
        } else {
            warn!("Git commit failed - no changes to commit");
        }
        
        Ok(())
    }
    
    async fn start_web_server(&self) -> Result<()> {
        info!("Starting web server for WASM hosting on port {}", self.args.port);
        
        let app = Router::new()
            .route("/", get(self.health_check))
            .route("/review", post(self.review_endpoint))
            .route("/wasm/analyze", post(self.wasm_analyze_endpoint))
            .route("/llm/analyze", post(self.llm_analyze_endpoint));
        
        let addr = format!("0.0.0.0:{}", self.args.port);
        info!("Web server starting on {}", addr);
        
        axum::Server::bind(&addr.parse()?)
            .serve(app.into_make_service())
            .await?;
        
        Ok(())
    }
    
    async fn health_check(&self) -> StatusCode {
        StatusCode::OK
    }
    
    async fn review_endpoint(&self, Json(payload): Json<serde_json::Value>) -> Json<serde_json::Value> {
        // Handle review requests via web API
        Json(serde_json::json!({
            "status": "success",
            "message": "Review endpoint ready"
        }))
    }
    
    async fn wasm_analyze_endpoint(&self, Json(payload): Json<serde_json::Value>) -> Json<serde_json::Value> {
        // Handle WASM analysis requests
        Json(serde_json::json!({
            "status": "success",
            "wasm_analysis": "ready"
        }))
    }
    
    async fn llm_analyze_endpoint(&self, Json(payload): Json<serde_json::Value>) -> Json<serde_json::Value> {
        // Handle LLM analysis requests
        Json(serde_json::json!({
            "status": "success",
            "llm_analysis": "ready"
        }))
    }
    
    async fn run_interactive_mode(&self) -> Result<()> {
        info!("Starting interactive mode with WASM and LLM capabilities...");
        
        loop {
            println!("\nDevAgent Interactive Mode (Rust + WASM + LLM)");
            println!("1. Review codebase");
            println!("2. WASM analysis");
            println!("3. LLM analysis");
            println!("4. Memory operations");
            println!("5. Start web server");
            println!("6. Exit");
            print!("Choose an option: ");
            
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            
            match input.trim() {
                "1" => {
                    let reviews = self.review_codebase().await?;
                    self.save_reviews(&reviews).await?;
                    println!("Code review completed!");
                }
                "2" => {
                    println!("WASM analysis mode - analyzing Rust files for WASM compilation...");
                    // WASM analysis logic
                }
                "3" => {
                    println!("LLM analysis mode - AI-powered code analysis...");
                    // LLM analysis logic
                }
                "4" => {
                    println!("Memory operations - managing code context...");
                    // Memory operations
                }
                "5" => {
                    println!("Starting web server...");
                    self.start_web_server().await?;
                }
                "6" => break,
                _ => println!("Invalid option"),
            }
        }
        
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    
    // Initialize logging
    if args.verbose {
        tracing_subscriber::fmt()
            .with_env_filter("debug")
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_env_filter("info")
            .init();
    }
    
    info!("Starting DevAgent Pipeline v0.1.0 (Rust + WASM + LLM)");
    
    let agent = DevAgent::new(args.clone()).await?;
    
    if args.web {
        agent.start_web_server().await?;
    } else if args.interactive {
        agent.run_interactive_mode().await?;
    } else {
        // Run automated review
        let reviews = agent.review_codebase().await?;
        
        // Save results
        agent.save_reviews(&reviews).await?;
        
        // Generate patches
        agent.generate_patches(&reviews).await?;
        
        // Optionally commit changes
        if !reviews.is_empty() {
            agent.commit_changes().await?;
        }
        
        info!("DevAgent pipeline completed successfully!");
        
        // Print summary
        let total_issues: usize = reviews.iter()
            .map(|r| r.issues.len())
            .sum();
        let total_suggestions: usize = reviews.iter()
            .map(|r| r.suggestions.len())
            .sum();
        
        println!("\n=== Review Summary ===");
        println!("Files reviewed: {}", reviews.len());
        println!("Total issues found: {}", total_issues);
        println!("Total suggestions: {}", total_suggestions);
        println!("Average score: {:.2}", 
            reviews.iter().map(|r| r.score).sum::<f32>() / reviews.len() as f32);
    }
    
    Ok(())
} 