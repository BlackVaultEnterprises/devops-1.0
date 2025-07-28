use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tracing::{info, warn, error};

// Local LLM integration
use kalosm::language::*;
use kalosm::*;

// MCP server integration for cloud delegation
use agentai::mcp::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct LocalBrainConfig {
    pub model_path: PathBuf,
    pub max_tokens: usize,
    pub temperature: f32,
    pub gpu_enabled: bool,
    pub mcp_servers: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VoiceCommand {
    pub text: String,
    pub confidence: f32,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub context: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BrainResponse {
    pub action: BrainAction,
    pub confidence: f32,
    pub reasoning: String,
    pub requires_cloud: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum BrainAction {
    LocalExecution(String),
    CloudDelegation(String),
    VoiceResponse(String),
    FileOperation(String),
    GitOperation(String),
    BuildOperation(String),
    TestOperation(String),
    WebSearch(String),
    CodeAnalysis(String),
    NoAction,
}

pub struct LocalBrain {
    config: LocalBrainConfig,
    phi_model: Arc<Mutex<Option<Phi3MiniInstruct>>>,
    mcp_client: Arc<Mutex<MCPClient>>,
    command_history: Arc<Mutex<Vec<VoiceCommand>>>,
}

impl LocalBrain {
    pub async fn new(config: LocalBrainConfig) -> Result<Self> {
        info!("Initializing Local Brain with Phi-3-mini-instruct");
        
        // Initialize Phi-3-mini-instruct model
        let phi_model = if config.gpu_enabled {
            info!("Loading Phi-3-mini-instruct with GPU acceleration");
            let model = Phi3MiniInstruct::builder()
                .with_source(Phi3MiniInstructSource::Local(config.model_path))
                .build()
                .await?;
            Arc::new(Mutex::new(Some(model)))
        } else {
            info!("Loading Phi-3-mini-instruct with CPU");
            let model = Phi3MiniInstruct::builder()
                .with_source(Phi3MiniInstructSource::Local(config.model_path))
                .build()
                .await?;
            Arc::new(Mutex::new(Some(model)))
        };
        
        // Initialize MCP client for cloud delegation
        let mcp_client = Arc::new(Mutex::new(MCPClient::new()));
        
        Ok(Self {
            config,
            phi_model,
            mcp_client,
            command_history: Arc::new(Mutex::new(Vec::new())),
        })
    }
    
    pub async fn process_voice_command(&self, command: VoiceCommand) -> Result<BrainResponse> {
        info!("Processing voice command: {}", command.text);
        
        // Store command in history
        {
            let mut history = self.command_history.lock().await;
            history.push(command.clone());
            if history.len() > 100 {
                history.remove(0);
            }
        }
        
        // Analyze command with local brain
        let response = self.analyze_command(&command).await?;
        
        // Execute action based on response
        self.execute_action(&response).await?;
        
        Ok(response)
    }
    
    async fn analyze_command(&self, command: &VoiceCommand) -> Result<BrainResponse> {
        let prompt = self.build_analysis_prompt(command);
        
        let model_guard = self.phi_model.lock().await;
        if let Some(model) = &*model_guard {
            let response = model.generate_text(&prompt).await?;
            self.parse_brain_response(&response)
        } else {
            Err(anyhow::anyhow!("Phi-3 model not loaded"))
        }
    }
    
    fn build_analysis_prompt(&self, command: &VoiceCommand) -> String {
        let context = self.get_recent_context().await;
        
        format!(
            r#"You are a local AI brain that processes voice commands for a developer environment.

Recent context: {}
Current command: "{}"

Analyze this command and determine:
1. Can this be executed locally or does it need cloud delegation?
2. What specific action should be taken?
3. What reasoning supports this decision?

Respond in JSON format:
{{
    "action": "local_execution|cloud_delegation|voice_response|file_operation|git_operation|build_operation|test_operation|web_search|code_analysis|no_action",
    "confidence": 0.0-1.0,
    "reasoning": "explanation",
    "requires_cloud": true/false,
    "details": "specific action details"
}}"#,
            context,
            command.text
        )
    }
    
    async fn get_recent_context(&self) -> String {
        let history = self.command_history.lock().await;
        let recent: Vec<String> = history
            .iter()
            .rev()
            .take(5)
            .map(|cmd| cmd.text.clone())
            .collect();
        recent.join("; ")
    }
    
    fn parse_brain_response(&self, response: &str) -> Result<BrainResponse> {
        // Try to parse JSON response
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(response) {
            let action_str = json["action"].as_str().unwrap_or("no_action");
            let confidence = json["confidence"].as_f64().unwrap_or(0.5) as f32;
            let reasoning = json["reasoning"].as_str().unwrap_or("").to_string();
            let requires_cloud = json["requires_cloud"].as_bool().unwrap_or(false);
            
            let action = match action_str {
                "local_execution" => BrainAction::LocalExecution(json["details"].as_str().unwrap_or("").to_string()),
                "cloud_delegation" => BrainAction::CloudDelegation(json["details"].as_str().unwrap_or("").to_string()),
                "voice_response" => BrainAction::VoiceResponse(json["details"].as_str().unwrap_or("").to_string()),
                "file_operation" => BrainAction::FileOperation(json["details"].as_str().unwrap_or("").to_string()),
                "git_operation" => BrainAction::GitOperation(json["details"].as_str().unwrap_or("").to_string()),
                "build_operation" => BrainAction::BuildOperation(json["details"].as_str().unwrap_or("").to_string()),
                "test_operation" => BrainAction::TestOperation(json["details"].as_str().unwrap_or("").to_string()),
                "web_search" => BrainAction::WebSearch(json["details"].as_str().unwrap_or("").to_string()),
                "code_analysis" => BrainAction::CodeAnalysis(json["details"].as_str().unwrap_or("").to_string()),
                _ => BrainAction::NoAction,
            };
            
            Ok(BrainResponse {
                action,
                confidence,
                reasoning,
                requires_cloud,
            })
        } else {
            // Fallback parsing for non-JSON responses
            let action = if response.to_lowercase().contains("cloud") {
                BrainAction::CloudDelegation(response.to_string())
            } else {
                BrainAction::LocalExecution(response.to_string())
            };
            
            Ok(BrainResponse {
                action,
                confidence: 0.7,
                reasoning: "Fallback parsing".to_string(),
                requires_cloud: matches!(action, BrainAction::CloudDelegation(_)),
            })
        }
    }
    
    async fn execute_action(&self, response: &BrainResponse) -> Result<()> {
        match &response.action {
            BrainAction::LocalExecution(details) => {
                info!("Executing locally: {}", details);
                self.execute_local_command(details).await?;
            }
            BrainAction::CloudDelegation(details) => {
                info!("Delegating to cloud: {}", details);
                self.delegate_to_cloud(details).await?;
            }
            BrainAction::VoiceResponse(message) => {
                info!("Generating voice response: {}", message);
                // TODO: Integrate with voice synthesis
            }
            BrainAction::FileOperation(operation) => {
                info!("File operation: {}", operation);
                self.execute_file_operation(operation).await?;
            }
            BrainAction::GitOperation(operation) => {
                info!("Git operation: {}", operation);
                self.execute_git_operation(operation).await?;
            }
            BrainAction::BuildOperation(operation) => {
                info!("Build operation: {}", operation);
                self.execute_build_operation(operation).await?;
            }
            BrainAction::TestOperation(operation) => {
                info!("Test operation: {}", operation);
                self.execute_test_operation(operation).await?;
            }
            BrainAction::WebSearch(query) => {
                info!("Web search: {}", query);
                self.execute_web_search(query).await?;
            }
            BrainAction::CodeAnalysis(path) => {
                info!("Code analysis: {}", path);
                self.execute_code_analysis(path).await?;
            }
            BrainAction::NoAction => {
                info!("No action required");
            }
        }
        
        Ok(())
    }
    
    async fn execute_local_command(&self, command: &str) -> Result<()> {
        // Execute local system commands
        let output = tokio::process::Command::new("cmd")
            .args(&["/C", command])
            .output()
            .await?;
        
        if output.status.success() {
            info!("Local command executed successfully");
        } else {
            warn!("Local command failed: {}", String::from_utf8_lossy(&output.stderr));
        }
        
        Ok(())
    }
    
    async fn delegate_to_cloud(&self, details: &str) -> Result<()> {
        let mut client = self.mcp_client.lock().await;
        
        // Connect to available MCP servers
        for server_url in &self.config.mcp_servers {
            if let Ok(_) = client.connect(server_url).await {
                info!("Connected to MCP server: {}", server_url);
                
                // Send command to cloud LLM via MCP
                let response = client.send_message(details).await?;
                info!("Cloud response: {}", response);
                break;
            }
        }
        
        Ok(())
    }
    
    async fn execute_file_operation(&self, operation: &str) -> Result<()> {
        // Parse file operation and execute
        if operation.contains("create") {
            // TODO: Implement file creation
        } else if operation.contains("delete") {
            // TODO: Implement file deletion
        } else if operation.contains("move") {
            // TODO: Implement file moving
        }
        
        Ok(())
    }
    
    async fn execute_git_operation(&self, operation: &str) -> Result<()> {
        // Execute git commands
        let output = tokio::process::Command::new("git")
            .args(operation.split_whitespace().collect::<Vec<_>>())
            .output()
            .await?;
        
        if output.status.success() {
            info!("Git operation completed");
        } else {
            warn!("Git operation failed: {}", String::from_utf8_lossy(&output.stderr));
        }
        
        Ok(())
    }
    
    async fn execute_build_operation(&self, operation: &str) -> Result<()> {
        // Execute build commands
        let output = tokio::process::Command::new("cargo")
            .args(operation.split_whitespace().collect::<Vec<_>>())
            .output()
            .await?;
        
        if output.status.success() {
            info!("Build operation completed");
        } else {
            warn!("Build operation failed: {}", String::from_utf8_lossy(&output.stderr));
        }
        
        Ok(())
    }
    
    async fn execute_test_operation(&self, operation: &str) -> Result<()> {
        // Execute test commands
        let output = tokio::process::Command::new("cargo")
            .args(&["test"])
            .output()
            .await?;
        
        if output.status.success() {
            info!("Test operation completed");
        } else {
            warn!("Test operation failed: {}", String::from_utf8_lossy(&output.stderr));
        }
        
        Ok(())
    }
    
    async fn execute_web_search(&self, query: &str) -> Result<()> {
        // TODO: Implement web search via MCP
        info!("Web search for: {}", query);
        Ok(())
    }
    
    async fn execute_code_analysis(&self, path: &str) -> Result<()> {
        // TODO: Implement code analysis
        info!("Code analysis for: {}", path);
        Ok(())
    }
} 