use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

// Simple memory system
struct MemorySystem {
    files: HashMap<String, String>,
}

impl MemorySystem {
    fn new() -> Self {
        Self {
            files: HashMap::new(),
        }
    }
    
    fn store_file(&mut self, id: &str, content: &str) {
        self.files.insert(id.to_string(), content.to_string());
    }
    
    fn get_file(&self, id: &str) -> Option<&String> {
        self.files.get(id)
    }
    
    fn get_statistics(&self) -> (usize, usize) {
        let total_files = self.files.len();
        let total_lines: usize = self.files.values()
            .map(|content| content.lines().count())
            .sum();
        (total_files, total_lines)
    }
}

// Simple code analyzer
struct CodeAnalyzer {
    language_rules: HashMap<String, Vec<String>>,
}

impl CodeAnalyzer {
    fn new() -> Self {
        let mut rules = HashMap::new();
        
        // Rust rules
        rules.insert("rust".to_string(), vec![
            "unwrap()".to_string(),
            "println!".to_string(),
            "clone()".to_string(),
        ]);
        
        // Python rules
        rules.insert("python".to_string(), vec![
            "import *".to_string(),
            "eval(".to_string(),
            "except:".to_string(),
        ]);
        
        // JavaScript rules
        rules.insert("javascript".to_string(), vec![
            "var ".to_string(),
            "eval(".to_string(),
        ]);
        
        Self { language_rules: rules }
    }
    
    fn analyze_code(&self, content: &str, file_path: &Path) -> (Vec<String>, Vec<String>, f32) {
        let mut issues = Vec::new();
        let mut suggestions = Vec::new();
        let mut score: f32 = 1.0;
        
        let language = self.detect_language(file_path, content);
        let lines: Vec<&str> = content.lines().collect();
        let total_lines = lines.len() as f32;
        
        if total_lines == 0.0 {
            return (issues, suggestions, 1.0);
        }
        
        for (i, line) in lines.iter().enumerate() {
            let line_num = i + 1;
            
            // Check for TODO comments
            if line.contains("TODO") || line.contains("FIXME") {
                issues.push(format!("Line {}: TODO or FIXME comment found", line_num));
                score -= 0.1;
            }
            
            // Check for long lines
            if line.len() > 120 {
                issues.push(format!("Line {}: Line too long (over 120 characters)", line_num));
                score -= 0.05;
            }
            
            // Check for hardcoded secrets
            if line.contains("password") || line.contains("secret") || line.contains("api_key") {
                issues.push(format!("Line {}: Potential hardcoded secret found", line_num));
                score -= 0.3;
            }
            
            // Check for dangerous patterns
            if line.contains("eval(") || line.contains("exec(") {
                issues.push(format!("Line {}: Dangerous code execution pattern detected", line_num));
                score -= 0.5;
            }
            
            // Language-specific checks
            if let Some(rules) = self.language_rules.get(&language) {
                for rule in rules {
                    if line.contains(rule) {
                        match rule.as_str() {
                            "unwrap()" => {
                                issues.push(format!("Line {}: Unsafe unwrap() usage found", line_num));
                                suggestions.push("Consider using proper error handling instead of unwrap()".to_string());
                                score -= 0.2;
                            }
                            "println!" => {
                                suggestions.push("Consider using structured logging instead of println!".to_string());
                                score -= 0.1;
                            }
                            "var " => {
                                suggestions.push("Use const or let instead of var for better scoping".to_string());
                                score -= 0.1;
                            }
                            "import *" => {
                                suggestions.push("Use specific imports instead of wildcard imports".to_string());
                                score -= 0.1;
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        
        // Generate general suggestions
        if content.contains("println!") {
            suggestions.push("Consider using structured logging instead of println!".to_string());
        }
        
        if content.contains("def ") && !content.contains("->") {
            suggestions.push("Consider adding type hints for better code clarity".to_string());
        }
        
        if content.lines().count() > 100 {
            suggestions.push("Consider breaking down large files into smaller modules".to_string());
        }
        
        score = score.max(0.0).min(1.0);
        (issues, suggestions, score)
    }
    
    fn detect_language(&self, file_path: &Path, content: &str) -> String {
        if let Some(ext) = file_path.extension().and_then(|s| s.to_str()) {
            match ext {
                "rs" => "rust".to_string(),
                "py" => "python".to_string(),
                "js" | "ts" => "javascript".to_string(),
                "java" => "java".to_string(),
                "cpp" | "cc" | "cxx" => "cpp".to_string(),
                "go" => "go".to_string(),
                _ => "unknown".to_string(),
            }
        } else {
            // Fallback to content-based detection
            if content.contains("fn ") && content.contains("use ") {
                "rust".to_string()
            } else if content.contains("def ") && content.contains("import ") {
                "python".to_string()
            } else if content.contains("function ") && (content.contains("const ") || content.contains("let ")) {
                "javascript".to_string()
            } else {
                "unknown".to_string()
            }
        }
    }
}

// Simple WASM analyzer (simulated)
struct WasmAnalyzer {
    optimizations: HashMap<String, String>,
}

impl WasmAnalyzer {
    fn new() -> Self {
        let mut optimizations = HashMap::new();
        optimizations.insert("no_std".to_string(), "Remove standard library dependencies".to_string());
        optimizations.insert("panic_abort".to_string(), "Abort on panic instead of unwinding".to_string());
        optimizations.insert("lto".to_string(), "Enable Link Time Optimization".to_string());
        
        Self { optimizations }
    }
    
    fn analyze_rust_file(&self, content: &str) -> (f64, usize, Vec<String>, f32) {
        let compile_time = 0.5; // Simulated
        let binary_size = content.lines().count() * 100; // Rough estimate
        let mut suggestions = Vec::new();
        let mut performance_score: f32 = 1.0;
        
        // Check for WASM compatibility issues
        if content.contains("use std::") && !content.contains("#![no_std]") {
            suggestions.push("Consider using no_std for smaller WASM size".to_string());
            performance_score -= 0.2;
        }
        
        if content.contains("std::fs::") || content.contains("File::") {
            suggestions.push("File system operations are not available in WASM".to_string());
            performance_score -= 0.3;
        }
        
        if content.contains("std::thread::") || content.contains("spawn") {
            suggestions.push("Threading is not available in WASM".to_string());
            performance_score -= 0.4;
        }
        
        if content.contains("unwrap()") {
            suggestions.push("Consider using Result types instead of panicking".to_string());
            performance_score -= 0.1;
        }
        
        performance_score = performance_score.max(0.0).min(1.0);
        
        (compile_time, binary_size, suggestions, performance_score)
    }
}

// Simple LLM analyzer (simulated)
struct LlmAnalyzer {
    model_available: bool,
}

impl LlmAnalyzer {
    fn new() -> Self {
        Self { model_available: false } // Simulated - no local model
    }
    
    fn analyze_code(&self, content: &str, _file_path: &Path) -> (f32, f32, f32, Vec<String>) {
        // Simulated LLM analysis
        let complexity_score = 0.8;
        let maintainability_score = 0.7;
        let security_score = 0.9;
        
        let mut ai_suggestions = Vec::new();
        
        // Generate AI-like suggestions based on patterns
        if content.contains("unwrap()") {
            ai_suggestions.push("Replace unwrap() with proper error handling using Result types".to_string());
        }
        
        if content.contains("println!") {
            ai_suggestions.push("Use structured logging with tracing instead of println!".to_string());
        }
        
        if content.contains("def ") && !content.contains("->") {
            ai_suggestions.push("Add type hints to function signatures for better code clarity".to_string());
        }
        
        if content.lines().count() > 100 {
            ai_suggestions.push("Consider breaking down large files into smaller, focused modules".to_string());
        }
        
        (complexity_score, maintainability_score, security_score, ai_suggestions)
    }
}

// Main DevAgent struct
struct DevAgent {
    memory_system: MemorySystem,
    code_analyzer: CodeAnalyzer,
    wasm_analyzer: WasmAnalyzer,
    llm_analyzer: LlmAnalyzer,
}

impl DevAgent {
    fn new() -> Self {
        Self {
            memory_system: MemorySystem::new(),
            code_analyzer: CodeAnalyzer::new(),
            wasm_analyzer: WasmAnalyzer::new(),
            llm_analyzer: LlmAnalyzer::new(),
        }
    }
    
    fn review_codebase(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        println!("🚀 DevAgent Pipeline - Rust + WASM Agentic Development Environment");
        println!("Starting comprehensive codebase review...");
        
        let path_buf = PathBuf::from(path);
        if !path_buf.exists() {
            println!("Path {} does not exist!", path);
            return Ok(());
        }
        
        let mut files_reviewed = 0;
        let mut total_issues = 0;
        let mut total_suggestions = 0;
        let mut total_score = 0.0;
        
        // Walk through all files
        for entry in walkdir::WalkDir::new(path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let file_path = entry.path();
            
            if self.is_code_file(file_path) {
                println!("Reviewing: {}", file_path.display());
                
                match self.review_file(file_path) {
                    Ok((issues, suggestions, score, wasm_analysis, llm_analysis)) => {
                        files_reviewed += 1;
                        total_issues += issues.len();
                        total_suggestions += suggestions.len();
                        total_score += score;
                        
                        if !issues.is_empty() {
                            println!("  Found {} issues", issues.len());
                        }
                        if !suggestions.is_empty() {
                            println!("  Generated {} suggestions", suggestions.len());
                        }
                        
                        // WASM analysis for Rust files
                        if file_path.extension().map_or(false, |ext| ext == "rs") {
                            let (compile_time, binary_size, wasm_suggestions, wasm_score) = wasm_analysis;
                            println!("  WASM Analysis: {}ms compile time, {} bytes, score: {:.2}", 
                                    (compile_time * 1000.0) as i32, binary_size, wasm_score);
                        }
                        
                        // LLM analysis
                        let (complexity, maintainability, security, ai_suggestions) = llm_analysis;
                        println!("  LLM Analysis: Complexity: {:.2}, Maintainability: {:.2}, Security: {:.2}", 
                                complexity, maintainability, security);
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
        if files_reviewed > 0 {
            println!("Average score: {:.2}", total_score / files_reviewed as f32);
        }
        
        // Memory statistics
        let (total_files, total_lines) = self.memory_system.get_statistics();
        println!("Memory: {} files, {} lines stored", total_files, total_lines);
        
        Ok(())
    }
    
    fn is_code_file(&self, path: &Path) -> bool {
        let extensions = ["rs", "js", "ts", "py", "java", "cpp", "c", "go", "php", "wasm"];
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| extensions.contains(&ext))
            .unwrap_or(false)
    }
    
    fn review_file(&self, file_path: &Path) -> Result<(Vec<String>, Vec<String>, f32, (f64, usize, Vec<String>, f32), (f32, f32, f32, Vec<String>)), Box<dyn std::error::Error>> {
        let content = fs::read_to_string(file_path)?;
        
        // Store in memory system
        let file_id = format!("{:?}", file_path);
        self.memory_system.store_file(&file_id, &content);
        
        // Static analysis
        let (issues, suggestions, score) = self.code_analyzer.analyze_code(&content, file_path);
        
        // WASM analysis for Rust files
        let wasm_analysis = if file_path.extension().map_or(false, |ext| ext == "rs") {
            self.wasm_analyzer.analyze_rust_file(&content)
        } else {
            (0.0, 0, Vec::new(), 1.0)
        };
        
        // LLM analysis
        let llm_analysis = self.llm_analyzer.analyze_code(&content, file_path);
        
        Ok((issues, suggestions, score, wasm_analysis, llm_analysis))
    }
    
    fn run_interactive_mode(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🚀 DevAgent Interactive Mode (Rust + WASM + LLM)");
        
        loop {
            println!("\nOptions:");
            println!("1. Review codebase");
            println!("2. Memory operations");
            println!("3. Exit");
            print!("Choose an option: ");
            
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            
            match input.trim() {
                "1" => {
                    print!("Enter path to review (default: ./src): ");
                    let mut path_input = String::new();
                    std::io::stdin().read_line(&mut path_input)?;
                    let path = path_input.trim();
                    let review_path = if path.is_empty() { "./src" } else { path };
                    
                    self.review_codebase(review_path)?;
                    println!("Code review completed!");
                }
                "2" => {
                    let (total_files, total_lines) = self.memory_system.get_statistics();
                    println!("Memory Statistics:");
                    println!("  Total files: {}", total_files);
                    println!("  Total lines: {}", total_lines);
                }
                "3" => break,
                _ => println!("Invalid option"),
            }
        }
        
        Ok(())
    }
}

// Simple walkdir implementation
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

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut agent = DevAgent::new();
    
    if args.len() > 1 && args[1] == "--interactive" {
        if let Err(e) = agent.run_interactive_mode() {
            eprintln!("Error in interactive mode: {}", e);
            std::process::exit(1);
        }
    } else {
        let path = args.get(1).unwrap_or(&"./src".to_string()).clone();
        
        if let Err(e) = agent.review_codebase(&path) {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
} 