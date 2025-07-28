use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tracing::{info, warn, error};
use reqwest::Client;
use tokio::fs;

#[derive(Debug, Serialize, Deserialize)]
pub struct LlmAnalysis {
    pub complexity_score: f32,
    pub maintainability_score: f32,
    pub security_score: f32,
    pub ai_suggestions: Vec<String>,
    pub code_quality_metrics: CodeQualityMetrics,
    pub refactoring_suggestions: Vec<RefactoringSuggestion>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CodeQualityMetrics {
    pub cyclomatic_complexity: f32,
    pub lines_of_code: usize,
    pub comment_ratio: f32,
    pub function_count: usize,
    pub average_function_length: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RefactoringSuggestion {
    pub title: String,
    pub description: String,
    pub priority: String,
    pub code_example: String,
    pub impact: String,
}

pub struct LlmAgent {
    client: Client,
    model_endpoint: String,
    local_model_available: bool,
}

impl LlmAgent {
    pub async fn new() -> Result<Self> {
        info!("Initializing LLM Agent...");
        
        let client = Client::new();
        let model_endpoint = std::env::var("LLM_ENDPOINT")
            .unwrap_or_else(|_| "http://localhost:11434".to_string());
        
        // Check if local model is available
        let local_model_available = Self::check_local_model(&client, &model_endpoint).await;
        
        Ok(Self {
            client,
            model_endpoint,
            local_model_available,
        })
    }
    
    async fn check_local_model(client: &Client, endpoint: &str) -> bool {
        match client.get(&format!("{}/api/tags", endpoint)).send().await {
            Ok(response) => response.status().is_success(),
            Err(_) => false,
        }
    }
    
    pub async fn analyze_code(&self, content: &str, file_path: &Path) -> Result<LlmAnalysis> {
        info!("Analyzing code with LLM: {}", file_path.display());
        
        // Static analysis first
        let metrics = self.calculate_code_metrics(content);
        
        // Try local LLM first, fallback to static analysis
        let ai_suggestions = if self.local_model_available {
            self.get_ai_suggestions(content, file_path).await.unwrap_or_else(|_| {
                warn!("Local LLM failed, using static analysis");
                self.get_static_suggestions(content, file_path)
            })
        } else {
            self.get_static_suggestions(content, file_path)
        };
        
        let refactoring_suggestions = self.generate_refactoring_suggestions(content, &metrics);
        
        let complexity_score = self.calculate_complexity_score(&metrics);
        let maintainability_score = self.calculate_maintainability_score(&metrics);
        let security_score = self.calculate_security_score(content);
        
        Ok(LlmAnalysis {
            complexity_score,
            maintainability_score,
            security_score,
            ai_suggestions,
            code_quality_metrics: metrics,
            refactoring_suggestions,
        })
    }
    
    async fn get_ai_suggestions(&self, content: &str, file_path: &Path) -> Result<Vec<String>> {
        let prompt = format!(
            "Analyze this {} code and provide specific improvement suggestions:\n\n{}\n\nProvide 3-5 specific, actionable suggestions for improving code quality, performance, and maintainability.",
            file_path.extension().and_then(|s| s.to_str()).unwrap_or("unknown"),
            content
        );
        
        let request_body = serde_json::json!({
            "model": "phi-3-mini-instruct",
            "prompt": prompt,
            "stream": false,
            "options": {
                "temperature": 0.3,
                "top_p": 0.9,
                "max_tokens": 500
            }
        });
        
        let response = self.client
            .post(&format!("{}/api/generate", self.model_endpoint))
            .json(&request_body)
            .send()
            .await?;
        
        if response.status().is_success() {
            let response_json: serde_json::Value = response.json().await?;
            let response_text = response_json["response"].as_str().unwrap_or("");
            
            // Parse suggestions from response
            let suggestions: Vec<String> = response_text
                .lines()
                .filter(|line| line.trim().starts_with('-') || line.trim().starts_with('*'))
                .map(|line| line.trim_start_matches('-').trim_start_matches('*').trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            
            Ok(suggestions)
        } else {
            Err(anyhow::anyhow!("LLM request failed"))
        }
    }
    
    fn get_static_suggestions(&self, content: &str, file_path: &Path) -> Vec<String> {
        let mut suggestions = Vec::new();
        
        // Language-specific suggestions
        if let Some(ext) = file_path.extension().and_then(|s| s.to_str()) {
            match ext {
                "rs" => {
                    if content.contains("unwrap()") {
                        suggestions.push("Replace unwrap() with proper error handling using Result types".to_string());
                    }
                    if content.contains("println!") {
                        suggestions.push("Use structured logging with tracing instead of println!".to_string());
                    }
                    if content.contains("clone()") && content.matches("clone()").count() > 3 {
                        suggestions.push("Consider using references or more efficient data structures to reduce cloning".to_string());
                    }
                }
                "py" => {
                    if content.contains("def ") && !content.contains("->") {
                        suggestions.push("Add type hints to function signatures for better code clarity".to_string());
                    }
                    if content.contains("import *") {
                        suggestions.push("Use specific imports instead of wildcard imports".to_string());
                    }
                }
                "js" | "ts" => {
                    if content.contains("var ") {
                        suggestions.push("Use const or let instead of var for better scoping".to_string());
                    }
                    if content.contains("function ") && !content.contains("=>") {
                        suggestions.push("Consider using arrow functions for consistency".to_string());
                    }
                }
                _ => {}
            }
        }
        
        // General suggestions
        if content.lines().count() > 100 {
            suggestions.push("Consider breaking down large files into smaller, focused modules".to_string());
        }
        
        if content.matches("TODO").count() > 0 {
            suggestions.push("Address TODO comments to improve code completeness".to_string());
        }
        
        suggestions
    }
    
    fn calculate_code_metrics(&self, content: &str) -> CodeQualityMetrics {
        let lines: Vec<&str> = content.lines().collect();
        let lines_of_code = lines.len();
        
        let comment_lines = lines.iter()
            .filter(|line| line.trim().starts_with("//") || line.trim().starts_with("/*") || line.trim().starts_with("*"))
            .count();
        
        let comment_ratio = if lines_of_code > 0 {
            comment_lines as f32 / lines_of_code as f32
        } else {
            0.0
        };
        
        let function_count = content.matches("fn ").count() + content.matches("def ").count() + content.matches("function ").count();
        
        let average_function_length = if function_count > 0 {
            lines_of_code as f32 / function_count as f32
        } else {
            0.0
        };
        
        // Simple cyclomatic complexity estimation
        let complexity_indicators = content.matches("if ").count() + 
                                  content.matches("for ").count() + 
                                  content.matches("while ").count() + 
                                  content.matches("match ").count() + 
                                  content.matches("&&").count() + 
                                  content.matches("||").count();
        
        let cyclomatic_complexity = 1.0 + complexity_indicators as f32;
        
        CodeQualityMetrics {
            cyclomatic_complexity,
            lines_of_code,
            comment_ratio,
            function_count,
            average_function_length,
        }
    }
    
    fn calculate_complexity_score(&self, metrics: &CodeQualityMetrics) -> f32 {
        let mut score = 1.0;
        
        // Penalize high cyclomatic complexity
        if metrics.cyclomatic_complexity > 10.0 {
            score -= 0.3;
        } else if metrics.cyclomatic_complexity > 5.0 {
            score -= 0.1;
        }
        
        // Penalize very long functions
        if metrics.average_function_length > 50.0 {
            score -= 0.2;
        }
        
        // Bonus for good comment ratio
        if metrics.comment_ratio > 0.1 && metrics.comment_ratio < 0.3 {
            score += 0.1;
        }
        
        score.max(0.0).min(1.0)
    }
    
    fn calculate_maintainability_score(&self, metrics: &CodeQualityMetrics) -> f32 {
        let mut score = 1.0;
        
        // Penalize very large files
        if metrics.lines_of_code > 500 {
            score -= 0.4;
        } else if metrics.lines_of_code > 200 {
            score -= 0.2;
        }
        
        // Penalize too many functions in one file
        if metrics.function_count > 20 {
            score -= 0.3;
        }
        
        // Bonus for good structure
        if metrics.comment_ratio > 0.05 {
            score += 0.1;
        }
        
        score.max(0.0).min(1.0)
    }
    
    fn calculate_security_score(&self, content: &str) -> f32 {
        let mut score = 1.0;
        
        // Security issues to check
        let security_patterns = [
            ("password", 0.3),
            ("secret", 0.3),
            ("api_key", 0.4),
            ("token", 0.2),
            ("eval(", 0.5),
            ("exec(", 0.5),
            ("sql", 0.2),
        ];
        
        for (pattern, penalty) in security_patterns {
            if content.to_lowercase().contains(pattern) {
                score -= penalty;
            }
        }
        
        score.max(0.0).min(1.0)
    }
    
    fn generate_refactoring_suggestions(&self, content: &str, metrics: &CodeQualityMetrics) -> Vec<RefactoringSuggestion> {
        let mut suggestions = Vec::new();
        
        if metrics.cyclomatic_complexity > 10.0 {
            suggestions.push(RefactoringSuggestion {
                title: "Reduce Cyclomatic Complexity".to_string(),
                description: "Break down complex functions into smaller, more focused functions".to_string(),
                priority: "High".to_string(),
                code_example: "// Extract helper functions to reduce complexity".to_string(),
                impact: "High".to_string(),
            });
        }
        
        if metrics.average_function_length > 50.0 {
            suggestions.push(RefactoringSuggestion {
                title: "Extract Long Functions".to_string(),
                description: "Split long functions into smaller, more readable functions".to_string(),
                priority: "Medium".to_string(),
                code_example: "// Break function into smaller, focused functions".to_string(),
                impact: "Medium".to_string(),
            });
        }
        
        if content.matches("unwrap()").count() > 0 {
            suggestions.push(RefactoringSuggestion {
                title: "Improve Error Handling".to_string(),
                description: "Replace unwrap() calls with proper error handling".to_string(),
                priority: "High".to_string(),
                code_example: "// Use Result types and proper error handling".to_string(),
                impact: "High".to_string(),
            });
        }
        
        suggestions
    }
} 