use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    println!("DevAgent Pipeline v0.1.0 (Rust)");
    
    let args: Vec<String> = env::args().collect();
    let path = args.get(1).unwrap_or(&"./src".to_string()).clone();
    
    println!("Reviewing code in: {}", path);
    
    if let Err(e) = review_codebase(&path) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn review_codebase(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let path_buf = PathBuf::from(path);
    
    if !path_buf.exists() {
        println!("Path {} does not exist!", path);
        return Ok(());
    }
    
    let mut files_reviewed = 0;
    let mut total_issues = 0;
    let mut total_suggestions = 0;
    
    // Walk through all files
    for entry in walkdir::WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let file_path = entry.path();
        
        if is_code_file(file_path) {
            println!("Reviewing: {}", file_path.display());
            
            match review_file(file_path) {
                Ok((issues, suggestions)) => {
                    files_reviewed += 1;
                    total_issues += issues.len();
                    total_suggestions += suggestions.len();
                    
                    if !issues.is_empty() {
                        println!("  Found {} issues", issues.len());
                    }
                    if !suggestions.is_empty() {
                        println!("  Generated {} suggestions", suggestions.len());
                    }
                }
                Err(e) => {
                    eprintln!("  Error reviewing {}: {}", file_path.display(), e);
                }
            }
        }
    }
    
    println!("\n=== Review Summary ===");
    println!("Files reviewed: {}", files_reviewed);
    println!("Total issues found: {}", total_issues);
    println!("Total suggestions: {}", total_suggestions);
    
    Ok(())
}

fn is_code_file(path: &Path) -> bool {
    let extensions = ["rs", "js", "ts", "py", "java", "cpp", "c", "go", "php"];
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| extensions.contains(&ext))
        .unwrap_or(false)
}

fn review_file(file_path: &Path) -> Result<(Vec<String>, Vec<String>), Box<dyn std::error::Error>> {
    let content = fs::read_to_string(file_path)?;
    let mut issues = Vec::new();
    let mut suggestions = Vec::new();
    
    let lines: Vec<&str> = content.lines().collect();
    
    for (i, line) in lines.iter().enumerate() {
        let line_num = i + 1;
        
        // Check for TODO comments
        if line.contains("TODO") || line.contains("FIXME") {
            issues.push(format!("Line {}: TODO or FIXME comment found", line_num));
        }
        
        // Check for long lines
        if line.len() > 120 {
            issues.push(format!("Line {}: Line too long (over 120 characters)", line_num));
        }
        
        // Check for hardcoded secrets
        if line.contains("\"password\"") || line.contains("\"secret\"") {
            issues.push(format!("Line {}: Potential hardcoded secret found", line_num));
        }
        
        // Check for unwrap() in Rust
        if line.contains(".unwrap()") && file_path.extension().map_or(false, |ext| ext == "rs") {
            issues.push(format!("Line {}: Unsafe unwrap() usage found", line_num));
            suggestions.push("Consider using proper error handling instead of unwrap()".to_string());
        }
    }
    
    // Generate suggestions based on code patterns
    if content.contains("println!") && file_path.extension().map_or(false, |ext| ext == "rs") {
        suggestions.push("Consider using structured logging instead of println!".to_string());
    }
    
    if content.contains("def ") && file_path.extension().map_or(false, |ext| ext == "py") {
        suggestions.push("Consider adding type hints for better code clarity".to_string());
    }
    
    Ok((issues, suggestions))
}

// Simple implementation of walkdir functionality
mod walkdir {
    use std::fs;
    use std::path::Path;
    
    pub struct WalkDir {
        root: String,
        stack: Vec<String>,
    }
    
    impl WalkDir {
        pub fn new<P: AsRef<Path>>(path: P) -> Self {
            Self {
                root: path.as_ref().to_string_lossy().to_string(),
                stack: vec![path.as_ref().to_string_lossy().to_string()],
            }
        }
        
        pub fn into_iter(self) -> WalkDirIter {
            WalkDirIter { walk_dir: self }
        }
    }
    
    pub struct WalkDirIter {
        walk_dir: WalkDir,
    }
        
    impl Iterator for WalkDirIter {
        type Item = Result<DirEntry, std::io::Error>;
        
        fn next(&mut self) -> Option<Self::Item> {
            while let Some(path) = self.walk_dir.stack.pop() {
                if let Ok(metadata) = fs::metadata(&path) {
                    let is_dir = metadata.is_dir();
                    let entry = DirEntry {
                        path,
                        metadata,
                    };
                    
                    if is_dir {
                        if let Ok(entries) = fs::read_dir(&entry.path) {
                            for entry_result in entries {
                                if let Ok(entry) = entry_result {
                                    self.walk_dir.stack.push(entry.path().to_string_lossy().to_string());
                                }
                            }
                        }
                    }
                    
                    return Some(Ok(entry));
                }
            }
            None
        }
    }
    
    pub struct DirEntry {
        pub path: String,
        pub metadata: fs::Metadata,
    }
    
    impl DirEntry {
        pub fn path(&self) -> &Path {
            Path::new(&self.path)
        }
        
        pub fn file_type(&self) -> FileType {
            FileType(self.metadata.file_type())
        }
    }
    
    pub struct FileType(fs::FileType);
    
    impl FileType {
        pub fn is_file(&self) -> bool {
            self.0.is_file()
        }
    }
} 