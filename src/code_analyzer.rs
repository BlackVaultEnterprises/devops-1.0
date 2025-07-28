use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tracing::{info, warn, error};

#[derive(Debug, Serialize, Deserialize)]
pub struct CodeAnalysis {
    pub issues: Vec<Issue>,
    pub suggestions: Vec<Suggestion>,
    pub metrics: CodeMetrics,
    pub score: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Issue {
    pub severity: Severity,
    pub message: String,
    pub line: Option<usize>,
    pub code: Option<String>,
    pub category: IssueCategory,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Suggestion {
    pub title: String,
    pub description: String,
    pub code: Option<String>,
    pub impact: Impact,
    pub category: SuggestionCategory,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CodeMetrics {
    pub lines_of_code: usize,
    pub comment_lines: usize,
    pub blank_lines: usize,
    pub function_count: usize,
    pub class_count: usize,
    pub cyclomatic_complexity: f32,
    pub maintainability_index: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Impact {
    Low,
    Medium,
    High,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum IssueCategory {
    Security,
    Performance,
    Maintainability,
    Style,
    Documentation,
    ErrorHandling,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum SuggestionCategory {
    Optimization,
    Refactoring,
    Documentation,
    Testing,
    Security,
    Performance,
}

pub struct CodeAnalyzer {
    language_rules: std::collections::HashMap<String, LanguageRules>,
}

#[derive(Debug)]
struct LanguageRules {
    file_extensions: Vec<String>,
    keywords: Vec<String>,
    anti_patterns: Vec<AntiPattern>,
    best_practices: Vec<BestPractice>,
}

#[derive(Debug)]
struct AntiPattern {
    pattern: String,
    message: String,
    severity: Severity,
    category: IssueCategory,
}

#[derive(Debug)]
struct BestPractice {
    pattern: String,
    suggestion: String,
    impact: Impact,
    category: SuggestionCategory,
}

impl CodeAnalyzer {
    pub async fn new() -> Result<Self> {
        info!("Initializing Code Analyzer...");
        
        let mut language_rules = std::collections::HashMap::new();
        
        // Rust rules
        language_rules.insert("rust".to_string(), LanguageRules {
            file_extensions: vec!["rs".to_string()],
            keywords: vec!["fn".to_string(), "use".to_string(), "mod".to_string()],
            anti_patterns: vec![
                AntiPattern {
                    pattern: "unwrap()".to_string(),
                    message: "Unsafe unwrap() usage".to_string(),
                    severity: Severity::High,
                    category: IssueCategory::ErrorHandling,
                },
                AntiPattern {
                    pattern: "println!".to_string(),
                    message: "Use structured logging instead of println!".to_string(),
                    severity: Severity::Medium,
                    category: IssueCategory::Style,
                },
                AntiPattern {
                    pattern: "clone()".to_string(),
                    message: "Excessive cloning detected".to_string(),
                    severity: Severity::Medium,
                    category: IssueCategory::Performance,
                },
            ],
            best_practices: vec![
                BestPractice {
                    pattern: "Result<".to_string(),
                    suggestion: "Good use of Result types".to_string(),
                    impact: Impact::High,
                    category: SuggestionCategory::ErrorHandling,
                },
                BestPractice {
                    pattern: "tracing::".to_string(),
                    suggestion: "Using structured logging".to_string(),
                    impact: Impact::Medium,
                    category: SuggestionCategory::Style,
                },
            ],
        });
        
        // Python rules
        language_rules.insert("python".to_string(), LanguageRules {
            file_extensions: vec!["py".to_string()],
            keywords: vec!["def".to_string(), "import".to_string(), "class".to_string()],
            anti_patterns: vec![
                AntiPattern {
                    pattern: "import *".to_string(),
                    message: "Wildcard imports should be avoided".to_string(),
                    severity: Severity::Medium,
                    category: IssueCategory::Style,
                },
                AntiPattern {
                    pattern: "eval(".to_string(),
                    message: "Dangerous eval() usage".to_string(),
                    severity: Severity::Critical,
                    category: IssueCategory::Security,
                },
                AntiPattern {
                    pattern: "except:".to_string(),
                    message: "Bare except clause".to_string(),
                    severity: Severity::High,
                    category: IssueCategory::ErrorHandling,
                },
            ],
            best_practices: vec![
                BestPractice {
                    pattern: "def ".to_string(),
                    suggestion: "Consider adding type hints".to_string(),
                    impact: Impact::Medium,
                    category: SuggestionCategory::Documentation,
                },
            ],
        });
        
        // JavaScript/TypeScript rules
        language_rules.insert("javascript".to_string(), LanguageRules {
            file_extensions: vec!["js".to_string(), "ts".to_string()],
            keywords: vec!["function".to_string(), "const".to_string(), "let".to_string()],
            anti_patterns: vec![
                AntiPattern {
                    pattern: "var ".to_string(),
                    message: "Use const or let instead of var".to_string(),
                    severity: Severity::Medium,
                    category: IssueCategory::Style,
                },
                AntiPattern {
                    pattern: "eval(".to_string(),
                    message: "Dangerous eval() usage".to_string(),
                    severity: Severity::Critical,
                    category: IssueCategory::Security,
                },
            ],
            best_practices: vec![
                BestPractice {
                    pattern: "const ".to_string(),
                    suggestion: "Good use of const for immutable values".to_string(),
                    impact: Impact::Medium,
                    category: SuggestionCategory::Style,
                },
            ],
        });
        
        Ok(Self { language_rules })
    }
    
    pub async fn analyze_code(&self, content: &str, file_path: &Path) -> Result<Vec<Issue>> {
        let language = self.detect_language(file_path, content);
        let mut issues = Vec::new();
        
        let lines: Vec<&str> = content.lines().collect();
        
        for (i, line) in lines.iter().enumerate() {
            let line_num = i + 1;
            
            // Check for general issues
            issues.extend(self.check_general_issues(line, line_num));
            
            // Check for language-specific issues
            if let Some(rules) = self.language_rules.get(&language) {
                issues.extend(self.check_language_specific_issues(line, line_num, rules));
            }
        }
        
        Ok(issues)
    }
    
    pub async fn generate_suggestions(&self, content: &str, file_path: &Path) -> Result<Vec<Suggestion>> {
        let language = self.detect_language(file_path, content);
        let mut suggestions = Vec::new();
        
        // Generate general suggestions
        suggestions.extend(self.generate_general_suggestions(content, file_path));
        
        // Generate language-specific suggestions
        if let Some(rules) = self.language_rules.get(&language) {
            suggestions.extend(self.generate_language_specific_suggestions(content, rules));
        }
        
        Ok(suggestions)
    }
    
    pub fn calculate_score(&self, content: &str) -> f32 {
        let lines: Vec<&str> = content.lines().collect();
        let total_lines = lines.len() as f32;
        
        if total_lines == 0.0 {
            return 1.0;
        }
        
        let mut score = 1.0;
        let mut issues = 0.0;
        
        for line in lines {
            // Penalize common issues
            if line.contains("TODO") || line.contains("FIXME") {
                issues += 1.0;
            }
            if line.contains("unwrap()") {
                issues += 1.0;
            }
            if line.contains("println!") {
                issues += 0.5;
            }
            if line.len() > 120 {
                issues += 0.3;
            }
            if line.contains("password") || line.contains("secret") {
                issues += 2.0; // High penalty for potential secrets
            }
        }
        
        // Bonus for good practices
        if content.contains("use tracing::") {
            score += 0.1;
        }
        if content.contains("Result<") {
            score += 0.1;
        }
        if content.contains("//") || content.contains("/*") {
            score += 0.05; // Bonus for comments
        }
        
        score -= (issues / total_lines) * 0.5;
        score.max(0.0).min(1.0)
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
    
    fn check_general_issues(&self, line: &str, line_num: usize) -> Vec<Issue> {
        let mut issues = Vec::new();
        
        // Check for TODO comments
        if line.contains("TODO") || line.contains("FIXME") {
            issues.push(Issue {
                severity: Severity::Medium,
                message: "TODO or FIXME comment found".to_string(),
                line: Some(line_num),
                code: Some(line.to_string()),
                category: IssueCategory::Documentation,
            });
        }
        
        // Check for long lines
        if line.len() > 120 {
            issues.push(Issue {
                severity: Severity::Low,
                message: "Line too long (over 120 characters)".to_string(),
                line: Some(line_num),
                code: Some(line.to_string()),
                category: IssueCategory::Style,
            });
        }
        
        // Check for potential secrets
        if line.contains("password") || line.contains("secret") || line.contains("api_key") {
            issues.push(Issue {
                severity: Severity::High,
                message: "Potential hardcoded secret found".to_string(),
                line: Some(line_num),
                code: Some(line.to_string()),
                category: IssueCategory::Security,
            });
        }
        
        // Check for dangerous patterns
        if line.contains("eval(") || line.contains("exec(") {
            issues.push(Issue {
                severity: Severity::Critical,
                message: "Dangerous code execution pattern detected".to_string(),
                line: Some(line_num),
                code: Some(line.to_string()),
                category: IssueCategory::Security,
            });
        }
        
        issues
    }
    
    fn check_language_specific_issues(&self, line: &str, line_num: usize, rules: &LanguageRules) -> Vec<Issue> {
        let mut issues = Vec::new();
        
        for anti_pattern in &rules.anti_patterns {
            if line.contains(&anti_pattern.pattern) {
                issues.push(Issue {
                    severity: anti_pattern.severity.clone(),
                    message: anti_pattern.message.clone(),
                    line: Some(line_num),
                    code: Some(line.to_string()),
                    category: anti_pattern.category.clone(),
                });
            }
        }
        
        issues
    }
    
    fn generate_general_suggestions(&self, content: &str, file_path: &Path) -> Vec<Suggestion> {
        let mut suggestions = Vec::new();
        
        // General suggestions based on file size
        if content.lines().count() > 100 {
            suggestions.push(Suggestion {
                title: "Break down large file".to_string(),
                description: "Consider splitting this large file into smaller, focused modules".to_string(),
                code: None,
                impact: Impact::Medium,
                category: SuggestionCategory::Refactoring,
            });
        }
        
        // Suggestions based on content patterns
        if content.matches("TODO").count() > 0 {
            suggestions.push(Suggestion {
                title: "Address TODO comments".to_string(),
                description: "Review and address TODO comments to improve code completeness".to_string(),
                code: None,
                impact: Impact::Medium,
                category: SuggestionCategory::Documentation,
            });
        }
        
        if content.lines().count() > 0 && content.matches("//").count() == 0 {
            suggestions.push(Suggestion {
                title: "Add documentation".to_string(),
                description: "Consider adding comments to explain complex logic".to_string(),
                code: Some("// Add meaningful comments here".to_string()),
                impact: Impact::Low,
                category: SuggestionCategory::Documentation,
            });
        }
        
        suggestions
    }
    
    fn generate_language_specific_suggestions(&self, content: &str, rules: &LanguageRules) -> Vec<Suggestion> {
        let mut suggestions = Vec::new();
        
        for best_practice in &rules.best_practices {
            if content.contains(&best_practice.pattern) {
                suggestions.push(Suggestion {
                    title: "Good practice detected".to_string(),
                    description: best_practice.suggestion.clone(),
                    code: None,
                    impact: best_practice.impact.clone(),
                    category: best_practice.category.clone(),
                });
            }
        }
        
        suggestions
    }
} 