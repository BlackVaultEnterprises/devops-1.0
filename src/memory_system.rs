use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use tokio::fs;
use tracing::{info, warn, error};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub id: String,
    pub file_path: String,
    pub content: String,
    pub analysis_results: Option<AnalysisResults>,
    pub metadata: MemoryMetadata,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AnalysisResults {
    pub code_metrics: CodeMetrics,
    pub issues: Vec<String>,
    pub suggestions: Vec<String>,
    pub wasm_analysis: Option<WasmAnalysisData>,
    pub llm_analysis: Option<LlmAnalysisData>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CodeMetrics {
    pub lines_of_code: usize,
    pub function_count: usize,
    pub complexity_score: f32,
    pub maintainability_score: f32,
    pub security_score: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WasmAnalysisData {
    pub binary_size: usize,
    pub performance_score: f32,
    pub optimization_suggestions: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LlmAnalysisData {
    pub complexity_score: f32,
    pub maintainability_score: f32,
    pub security_score: f32,
    pub ai_suggestions: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MemoryMetadata {
    pub file_size: usize,
    pub language: String,
    pub last_modified: DateTime<Utc>,
    pub tags: Vec<String>,
}

pub struct MemorySystem {
    memory_file: String,
    entries: HashMap<String, MemoryEntry>,
}

impl MemorySystem {
    pub async fn new() -> Result<Self> {
        info!("Initializing Memory System...");
        
        let memory_file = "dev_agent_memory.json".to_string();
        let mut entries = HashMap::new();
        
        // Load existing memory if available
        if Path::new(&memory_file).exists() {
            match fs::read_to_string(&memory_file).await {
                Ok(content) => {
                    match serde_json::from_str::<HashMap<String, MemoryEntry>>(&content) {
                        Ok(loaded_entries) => {
                            entries = loaded_entries;
                            info!("Loaded {} memory entries", entries.len());
                        }
                        Err(e) => {
                            warn!("Failed to parse memory file: {}", e);
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to read memory file: {}", e);
                }
            }
        }
        
        Ok(Self {
            memory_file,
            entries,
        })
    }
    
    pub async fn store_file(&mut self, file_id: &str, content: &str) -> Result<()> {
        info!("Storing file in memory: {}", file_id);
        
        let metadata = self.extract_metadata(content);
        
        let entry = MemoryEntry {
            id: file_id.to_string(),
            file_path: file_id.to_string(), // Will be updated when we have actual path
            content: content.to_string(),
            analysis_results: None,
            metadata,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        
        self.entries.insert(file_id.to_string(), entry);
        self.save_memory().await?;
        
        Ok(())
    }
    
    pub async fn update_analysis(&mut self, file_id: &str, analysis: AnalysisResults) -> Result<()> {
        if let Some(entry) = self.entries.get_mut(file_id) {
            entry.analysis_results = Some(analysis);
            entry.updated_at = Utc::now();
            self.save_memory().await?;
            info!("Updated analysis for file: {}", file_id);
        } else {
            warn!("File not found in memory: {}", file_id);
        }
        
        Ok(())
    }
    
    pub async fn get_file(&self, file_id: &str) -> Option<&MemoryEntry> {
        self.entries.get(file_id)
    }
    
    pub async fn search_files(&self, query: &str) -> Vec<&MemoryEntry> {
        let query_lower = query.to_lowercase();
        let mut results = Vec::new();
        
        for entry in self.entries.values() {
            if entry.content.to_lowercase().contains(&query_lower) ||
               entry.file_path.to_lowercase().contains(&query_lower) ||
               entry.metadata.tags.iter().any(|tag| tag.to_lowercase().contains(&query_lower)) {
                results.push(entry);
            }
        }
        
        results
    }
    
    pub async fn get_recent_files(&self, limit: usize) -> Vec<&MemoryEntry> {
        let mut entries: Vec<&MemoryEntry> = self.entries.values().collect();
        entries.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        entries.truncate(limit);
        entries
    }
    
    pub async fn get_files_by_language(&self, language: &str) -> Vec<&MemoryEntry> {
        self.entries.values()
            .filter(|entry| entry.metadata.language == language)
            .collect()
    }
    
    pub async fn get_files_with_issues(&self) -> Vec<&MemoryEntry> {
        self.entries.values()
            .filter(|entry| {
                if let Some(ref analysis) = entry.analysis_results {
                    !analysis.issues.is_empty()
                } else {
                    false
                }
            })
            .collect()
    }
    
    pub async fn get_statistics(&self) -> MemoryStatistics {
        let total_files = self.entries.len();
        let total_lines = self.entries.values()
            .map(|entry| entry.content.lines().count())
            .sum();
        
        let languages: std::collections::HashMap<String, usize> = self.entries.values()
            .fold(HashMap::new(), |mut acc, entry| {
                *acc.entry(entry.metadata.language.clone()).or_insert(0) += 1;
                acc
            });
        
        let files_with_issues = self.entries.values()
            .filter(|entry| {
                if let Some(ref analysis) = entry.analysis_results {
                    !analysis.issues.is_empty()
                } else {
                    false
                }
            })
            .count();
        
        MemoryStatistics {
            total_files,
            total_lines,
            languages,
            files_with_issues,
            memory_size_bytes: self.calculate_memory_size(),
        }
    }
    
    fn extract_metadata(&self, content: &str) -> MemoryMetadata {
        let file_size = content.len();
        let language = self.detect_language(content);
        let tags = self.extract_tags(content);
        
        MemoryMetadata {
            file_size,
            language,
            last_modified: Utc::now(),
            tags,
        }
    }
    
    fn detect_language(&self, content: &str) -> String {
        // Simple language detection based on file content patterns
        if content.contains("fn ") && content.contains("use ") {
            "rust".to_string()
        } else if content.contains("def ") && content.contains("import ") {
            "python".to_string()
        } else if content.contains("function ") && (content.contains("const ") || content.contains("let ")) {
            "javascript".to_string()
        } else if content.contains("public class ") || content.contains("public static void main") {
            "java".to_string()
        } else if content.contains("#include ") && content.contains("int main") {
            "cpp".to_string()
        } else if content.contains("package ") && content.contains("func ") {
            "go".to_string()
        } else {
            "unknown".to_string()
        }
    }
    
    fn extract_tags(&self, content: &str) -> Vec<String> {
        let mut tags = Vec::new();
        
        // Extract TODO tags
        for line in content.lines() {
            if line.contains("TODO") {
                tags.push("todo".to_string());
            }
            if line.contains("FIXME") {
                tags.push("fixme".to_string());
            }
            if line.contains("BUG") {
                tags.push("bug".to_string());
            }
        }
        
        // Extract function names as tags
        for line in content.lines() {
            if line.contains("fn ") {
                if let Some(func_name) = line.split("fn ").nth(1) {
                    if let Some(name) = func_name.split('(').next() {
                        tags.push(format!("fn:{}", name.trim()));
                    }
                }
            }
        }
        
        tags
    }
    
    async fn save_memory(&self) -> Result<()> {
        let json = serde_json::to_string_pretty(&self.entries)
            .context("Failed to serialize memory")?;
        
        fs::write(&self.memory_file, json).await
            .context("Failed to write memory file")?;
        
        Ok(())
    }
    
    fn calculate_memory_size(&self) -> usize {
        serde_json::to_string(&self.entries)
            .map(|s| s.len())
            .unwrap_or(0)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MemoryStatistics {
    pub total_files: usize,
    pub total_lines: usize,
    pub languages: std::collections::HashMap<String, usize>,
    pub files_with_issues: usize,
    pub memory_size_bytes: usize,
}

impl MemorySystem {
    pub async fn clear_memory(&mut self) -> Result<()> {
        info!("Clearing memory system...");
        self.entries.clear();
        self.save_memory().await?;
        Ok(())
    }
    
    pub async fn export_memory(&self, export_path: &str) -> Result<()> {
        info!("Exporting memory to: {}", export_path);
        
        let export_data = serde_json::to_string_pretty(&self.entries)
            .context("Failed to serialize memory for export")?;
        
        fs::write(export_path, export_data).await
            .context("Failed to write export file")?;
        
        Ok(())
    }
    
    pub async fn import_memory(&mut self, import_path: &str) -> Result<()> {
        info!("Importing memory from: {}", import_path);
        
        let content = fs::read_to_string(import_path).await
            .context("Failed to read import file")?;
        
        let imported_entries: HashMap<String, MemoryEntry> = serde_json::from_str(&content)
            .context("Failed to parse import file")?;
        
        for (key, entry) in imported_entries {
            self.entries.insert(key, entry);
        }
        
        self.save_memory().await?;
        info!("Imported {} entries", imported_entries.len());
        
        Ok(())
    }
} 