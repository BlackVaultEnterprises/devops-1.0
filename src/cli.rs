use clap::{Parser, Subcommand};
use std::path::PathBuf;
use anyhow::Result;
use tokio::process::Command;

#[derive(Parser)]
#[command(name = "kov-code-agent")]
#[command(about = "Kowalski Code Agent - AI-powered code review and refactoring")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Review code in a directory
    Review {
        /// Path to review
        #[arg(default_value = "./src")]
        path: PathBuf,
        
        /// Output file for results
        #[arg(short, long)]
        output: Option<PathBuf>,
        
        /// Enable verbose output
        #[arg(short, long)]
        verbose: bool,
    },
    
    /// Generate patches for suggested improvements
    Patch {
        /// Path to review
        #[arg(default_value = "./src")]
        path: PathBuf,
        
        /// Output directory for patches
        #[arg(short, long, default_value = "./patches")]
        output: PathBuf,
    },
    
    /// Commit changes automatically
    Commit {
        /// Commit message
        #[arg(short, long, default_value = "Auto-generated improvements from Kowalski Code Agent")]
        message: String,
        
        /// Review before committing
        #[arg(short, long)]
        review: bool,
    },
    
    /// Run interactive mode
    Interactive {
        /// Path to review
        #[arg(default_value = "./src")]
        path: PathBuf,
    },
}

pub async fn run_cli() -> Result<()> {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Review { path, output, verbose } => {
            println!("Reviewing code in: {}", path.display());
            
            // Run the review using our DevAgent
            let args = crate::Args {
                path,
                output,
                verbose,
                interactive: false,
            };
            
            let agent = crate::DevAgent::new(args).await?;
            let reviews = agent.review_codebase().await?;
            agent.save_reviews(&reviews).await?;
            
            println!("Review completed! Found {} files with issues.", reviews.len());
        }
        
        Commands::Patch { path, output } => {
            println!("Generating patches for: {}", path.display());
            
            let args = crate::Args {
                path,
                output: None,
                verbose: false,
                interactive: false,
            };
            
            let agent = crate::DevAgent::new(args).await?;
            let reviews = agent.review_codebase().await?;
            agent.generate_patches(&reviews).await?;
            
            println!("Patches generated in: {}", output.display());
        }
        
        Commands::Commit { message, review } => {
            if review {
                println!("Running review before commit...");
                let args = crate::Args {
                    path: PathBuf::from("./src"),
                    output: None,
                    verbose: false,
                    interactive: false,
                };
                
                let agent = crate::DevAgent::new(args).await?;
                let reviews = agent.review_codebase().await?;
                
                if !reviews.is_empty() {
                    println!("Found {} issues. Proceeding with commit...", 
                        reviews.iter().map(|r| r.issues.len()).sum::<usize>());
                }
            }
            
            println!("Committing changes with message: {}", message);
            
            let status = Command::new("git")
                .args(["add", "."])
                .status()
                .await?;
            
            if status.success() {
                let status = Command::new("git")
                    .args(["commit", "-m", &message])
                    .status()
                    .await?;
                
                if status.success() {
                    println!("Changes committed successfully!");
                } else {
                    println!("No changes to commit.");
                }
            } else {
                println!("Failed to add files to git.");
            }
        }
        
        Commands::Interactive { path } => {
            println!("Starting interactive mode for: {}", path.display());
            
            let args = crate::Args {
                path,
                output: None,
                verbose: false,
                interactive: true,
            };
            
            let agent = crate::DevAgent::new(args).await?;
            agent.run_interactive_mode().await?;
        }
    }
    
    Ok(())
}

// Helper function to run Kowalski CLI commands
pub async fn run_kowalski_command(args: &[&str]) -> Result<String> {
    let output = Command::new("kowalski")
        .args(args)
        .output()
        .await?;
    
    if output.status.success() {
        Ok(String::from_utf8(output.stdout)?)
    } else {
        Err(anyhow::anyhow!(
            "Kowalski command failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

// Helper function to check if Kowalski is available
pub async fn check_kowalski_available() -> bool {
    Command::new("kowalski")
        .arg("--version")
        .output()
        .await
        .map(|output| output.status.success())
        .unwrap_or(false)
} 