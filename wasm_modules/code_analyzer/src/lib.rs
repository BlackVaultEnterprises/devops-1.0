use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct CodeAnalysis {
    pub issues: Vec<String>,
    pub suggestions: Vec<String>,
    pub score: f32,
    pub metrics: CodeMetrics,
}

#[derive(Serialize, Deserialize)]
pub struct CodeMetrics {
    pub lines_of_code: usize,
    pub function_count: usize,
    pub cyclomatic_complexity: f32,
    pub comment_ratio: f32,
}

#[wasm_bindgen]
pub fn analyze_code(code: &str) -> Result<JsValue, JsValue> {
    let analysis = perform_analysis(code);
    Ok(serde_wasm_bindgen::to_value(&analysis)?)
}

#[wasm_bindgen]
pub fn optimize_code(code: &str) -> Result<JsValue, JsValue> {
    let optimized = perform_optimization(code);
    Ok(serde_wasm_bindgen::to_value(&optimized)?)
}

#[wasm_bindgen]
pub fn generate_suggestions(code: &str) -> Result<JsValue, JsValue> {
    let suggestions = generate_code_suggestions(code);
    Ok(serde_wasm_bindgen::to_value(&suggestions)?)
}

fn perform_analysis(code: &str) -> CodeAnalysis {
    let mut issues = Vec::new();
    let mut suggestions = Vec::new();
    let mut score = 1.0;
    
    let lines: Vec<&str> = code.lines().collect();
    let total_lines = lines.len() as f32;
    
    if total_lines == 0.0 {
        return CodeAnalysis {
            issues,
            suggestions,
            score: 1.0,
            metrics: CodeMetrics {
                lines_of_code: 0,
                function_count: 0,
                cyclomatic_complexity: 1.0,
                comment_ratio: 0.0,
            },
        };
    }
    
    let mut function_count = 0;
    let mut comment_lines = 0;
    let mut complexity_indicators = 0;
    
    for (i, line) in lines.iter().enumerate() {
        let line_num = i + 1;
        
        // Count functions
        if line.contains("fn ") || line.contains("def ") || line.contains("function ") {
            function_count += 1;
        }
        
        // Count comments
        if line.trim().starts_with("//") || line.trim().starts_with("/*") || line.trim().starts_with("*") {
            comment_lines += 1;
        }
        
        // Count complexity indicators
        if line.contains("if ") || line.contains("for ") || line.contains("while ") || 
           line.contains("match ") || line.contains("&&") || line.contains("||") {
            complexity_indicators += 1;
        }
        
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
        
        // Check for unwrap() in Rust
        if line.contains(".unwrap()") {
            issues.push(format!("Line {}: Unsafe unwrap() usage found", line_num));
            suggestions.push("Consider using proper error handling instead of unwrap()".to_string());
            score -= 0.2;
        }
        
        // Check for hardcoded secrets
        if line.contains("\"password\"") || line.contains("\"secret\"") || line.contains("\"api_key\"") {
            issues.push(format!("Line {}: Potential hardcoded secret found", line_num));
            score -= 0.3;
        }
        
        // Check for dangerous patterns
        if line.contains("eval(") || line.contains("exec(") {
            issues.push(format!("Line {}: Dangerous code execution pattern detected", line_num));
            score -= 0.5;
        }
    }
    
    // Generate suggestions based on code patterns
    if code.contains("println!") {
        suggestions.push("Consider using structured logging instead of println!".to_string());
    }
    
    if code.contains("def ") && !code.contains("->") {
        suggestions.push("Consider adding type hints for better code clarity".to_string());
    }
    
    if code.contains("var ") {
        suggestions.push("Use const or let instead of var for better scoping".to_string());
    }
    
    if code.contains("import *") {
        suggestions.push("Use specific imports instead of wildcard imports".to_string());
    }
    
    // Calculate metrics
    let comment_ratio = if total_lines > 0.0 {
        comment_lines as f32 / total_lines
    } else {
        0.0
    };
    
    let cyclomatic_complexity = 1.0 + complexity_indicators as f32;
    
    let metrics = CodeMetrics {
        lines_of_code: lines.len(),
        function_count,
        cyclomatic_complexity,
        comment_ratio,
    };
    
    // Adjust score based on metrics
    if cyclomatic_complexity > 10.0 {
        score -= 0.2;
    }
    
    if comment_ratio < 0.05 {
        score -= 0.1;
    }
    
    score = score.max(0.0).min(1.0);
    
    CodeAnalysis {
        issues,
        suggestions,
        score,
        metrics,
    }
}

fn perform_optimization(code: &str) -> String {
    let mut optimized = code.to_string();
    
    // Replace println! with tracing::info!
    optimized = optimized.replace("println!", "tracing::info!");
    
    // Replace unwrap() with proper error handling
    optimized = optimized.replace(".unwrap()", ".expect(\"Error message\")");
    
    // Replace var with const/let in JavaScript
    optimized = optimized.replace("var ", "const ");
    
    // Add type hints to Python functions
    let lines: Vec<&str> = optimized.lines().collect();
    let mut new_lines = Vec::new();
    
    for line in lines {
        if line.contains("def ") && !line.contains("->") && !line.contains(":") {
            // Simple type hint addition
            let new_line = line.replace("def ", "def ") + " -> None:";
            new_lines.push(new_line);
        } else {
            new_lines.push(line.to_string());
        }
    }
    
    new_lines.join("\n")
}

fn generate_code_suggestions(code: &str) -> Vec<String> {
    let mut suggestions = Vec::new();
    
    // Language-specific suggestions
    if code.contains("fn ") {
        suggestions.push("Consider adding documentation comments to functions".to_string());
        suggestions.push("Use Result types for better error handling".to_string());
    }
    
    if code.contains("def ") {
        suggestions.push("Add type hints to function signatures".to_string());
        suggestions.push("Consider using dataclasses for data structures".to_string());
    }
    
    if code.contains("function ") {
        suggestions.push("Consider using arrow functions for consistency".to_string());
        suggestions.push("Add JSDoc comments for better documentation".to_string());
    }
    
    // General suggestions
    if code.lines().count() > 100 {
        suggestions.push("Consider breaking down large files into smaller modules".to_string());
    }
    
    if code.matches("TODO").count() > 0 {
        suggestions.push("Address TODO comments to improve code completeness".to_string());
    }
    
    if code.contains("//") == false && code.lines().count() > 10 {
        suggestions.push("Add comments to explain complex logic".to_string());
    }
    
    suggestions
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_analyze_code() {
        let code = r#"
fn main() {
    println!("Hello, world!");
    let result = Some(42).unwrap();
}
"#;
        
        let analysis = perform_analysis(code);
        assert!(analysis.score < 1.0); // Should have issues
        assert!(!analysis.issues.is_empty());
    }
    
    #[test]
    fn test_optimize_code() {
        let code = "println!(\"test\");";
        let optimized = perform_optimization(code);
        assert!(optimized.contains("tracing::info!"));
        assert!(!optimized.contains("println!"));
    }
} 