use std::path::PathBuf;
use std::process::Command;
use anyhow::{Context, Result};
use clap::Parser;
use serde::{Deserialize, Serialize};
use tokio::fs;
use tracing::{info, warn, error};
use walkdir::WalkDir;

mod cli;

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
}

#[derive(Debug, Serialize, Deserialize)]
struct CodeReview {
    file_path: String,
    issues: Vec<Issue>,
    suggestions: Vec<Suggestion>,
    score: f32,
    timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Issue {
    severity: Severity,
    message: String,
    line: Option<usize>,
    code: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Suggestion {
    title: String,
    description: String,
    code: Option<String>,
    impact: Impact,
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
}

impl DevAgent {
    async fn new(args: Args) -> Result<Self> {
        info!("Initializing DevAgent...");
        Ok(Self { args })
    }
    
    async fn review_codebase(&self) -> Result<Vec<CodeReview>> {
        info!("Starting codebase review of: {}", self.args.path.display());
        
        let mut reviews = Vec::new();
        
        // Walk through the codebase
        for entry in WalkDir::new(&self.args.path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let file_path = entry.path();
            
            // Skip non-code files
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
        let extensions = ["rs", "js", "ts", "py", "java", "cpp", "c", "go", "php"];
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| extensions.contains(&ext))
            .unwrap_or(false)
    }
    
    async fn review_file(&self, file_path: &std::path::Path) -> Result<CodeReview> {
        let content = fs::read_to_string(file_path).await
            .context("Failed to read file")?;
        
        // Simple static analysis for demonstration
        let issues = self.analyze_code(&content)?;
        let suggestions = self.generate_suggestions(&content)?;
        let score = self.calculate_score(&content);
        
        Ok(CodeReview {
            file_path: file_path.to_string_lossy().to_string(),
            issues,
            suggestions,
            score,
            timestamp: chrono::Utc::now(),
        })
    }
    
    fn analyze_code(&self, content: &str) -> Result<Vec<Issue>> {
        let mut issues = Vec::new();
        
        // Simple static analysis rules
        let lines: Vec<&str> = content.lines().collect();
        
        for (i, line) in lines.iter().enumerate() {
            let line_num = i + 1;
            
            // Check for TODO comments
            if line.contains("TODO") || line.contains("FIXME") {
                issues.push(Issue {
                    severity: Severity::Medium,
                    message: "TODO or FIXME comment found".to_string(),
                    line: Some(line_num),
                    code: Some(line.to_string()),
                });
            }
            
            // Check for long lines
            if line.len() > 120 {
                issues.push(Issue {
                    severity: Severity::Low,
                    message: "Line too long (over 120 characters)".to_string(),
                    line: Some(line_num),
                    code: Some(line.to_string()),
                });
            }
            
            // Check for hardcoded strings
            if line.contains("\"password\"") || line.contains("\"secret\"") {
                issues.push(Issue {
                    severity: Severity::High,
                    message: "Potential hardcoded secret found".to_string(),
                    line: Some(line_num),
                    code: Some(line.to_string()),
                });
            }
        }
        
        Ok(issues)
    }
    
    fn generate_suggestions(&self, content: &str) -> Result<Vec<Suggestion>> {
        let mut suggestions = Vec::new();
        
        // Generate suggestions based on code patterns
        if content.contains("println!") {
            suggestions.push(Suggestion {
                title: "Use structured logging".to_string(),
                description: "Consider using a logging framework instead of println!".to_string(),
                code: Some("use tracing::{info, warn, error};".to_string()),
                impact: Impact::Medium,
            });
        }
        
        if content.contains("unwrap()") {
            suggestions.push(Suggestion {
                title: "Handle errors properly".to_string(),
                description: "Consider using proper error handling instead of unwrap()".to_string(),
                code: Some("// Use .map_err() or ? operator instead".to_string()),
                impact: Impact::High,
            });
        }
        
        Ok(suggestions)
    }
    
    fn calculate_score(&self, content: &str) -> f32 {
        let mut score = 1.0;
        
        // Simple scoring based on code quality indicators
        let lines: Vec<&str> = content.lines().collect();
        let total_lines = lines.len() as f32;
        
        let mut issues = 0.0;
        
        for line in lines {
            if line.contains("TODO") || line.contains("FIXME") {
                issues += 1.0;
            }
            if line.contains("unwrap()") {
                issues += 1.0;
            }
            if line.len() > 120 {
                issues += 0.5;
            }
        }
        
        score -= (issues / total_lines) * 0.5;
        score.max(0.0)
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
        info!("Generating patches for suggested improvements...");
        
        for review in reviews {
            for suggestion in &review.suggestions {
                if let Some(code) = &suggestion.code {
                    // Create a patch file
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
            .args(["commit", "-m", "Auto-generated code improvements from DevAgent"])
            .status()
            .context("Failed to git commit")?;
        
        if status.success() {
            info!("Changes committed successfully");
        } else {
            warn!("Git commit failed - no changes to commit");
        }
        
        Ok(())
    }
    
    async fn run_interactive_mode(&self) -> Result<()> {
        info!("Starting interactive mode...");
        
        loop {
            println!("\nDevAgent Interactive Mode");
            println!("1. Review codebase");
            println!("2. Generate patches");
            println!("3. Commit changes");
            println!("4. Exit");
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
                    let reviews = self.review_codebase().await?;
                    self.generate_patches(&reviews).await?;
                    println!("Patches generated!");
                }
                "3" => {
                    self.commit_changes().await?;
                    println!("Changes committed!");
                }
                "4" => break,
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
    
    info!("Starting DevAgent Pipeline v0.1.0");
    
    let agent = DevAgent::new(args.clone()).await?;
    
    if args.interactive {
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