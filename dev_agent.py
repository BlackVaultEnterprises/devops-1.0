#!/usr/bin/env python3
"""
DevAgent Pipeline - Python Implementation
A comprehensive agentic development environment with AI-powered code review
"""

import os
import sys
import json
import argparse
import subprocess
import glob
from pathlib import Path
from datetime import datetime
from typing import List, Dict, Any, Optional

class CodeReview:
    def __init__(self, file_path: str, issues: List[Dict], suggestions: List[Dict], score: float):
        self.file_path = file_path
        self.issues = issues
        self.suggestions = suggestions
        self.score = score
        self.timestamp = datetime.utcnow()

class DevAgent:
    def __init__(self, args):
        self.args = args
        self.code_extensions = ['.rs', '.js', '.ts', '.py', '.java', '.cpp', '.c', '.go', '.php']
        
    def is_code_file(self, file_path: Path) -> bool:
        return file_path.suffix.lower() in self.code_extensions
    
    def analyze_code(self, content: str, file_path: str) -> List[Dict]:
        issues = []
        lines = content.split('\n')
        
        for i, line in enumerate(lines, 1):
            # Check for TODO comments
            if 'TODO' in line or 'FIXME' in line:
                issues.append({
                    'severity': 'Medium',
                    'message': 'TODO or FIXME comment found',
                    'line': i,
                    'code': line.strip()
                })
            
            # Check for long lines
            if len(line) > 120:
                issues.append({
                    'severity': 'Low',
                    'message': 'Line too long (over 120 characters)',
                    'line': i,
                    'code': line.strip()
                })
            
            # Check for hardcoded secrets
            if '"password"' in line or '"secret"' in line:
                issues.append({
                    'severity': 'High',
                    'message': 'Potential hardcoded secret found',
                    'line': i,
                    'code': line.strip()
                })
            
            # Check for unwrap() in Rust
            if '.unwrap()' in line and file_path.endswith('.rs'):
                issues.append({
                    'severity': 'High',
                    'message': 'Unsafe unwrap() usage found',
                    'line': i,
                    'code': line.strip()
                })
        
        return issues
    
    def generate_suggestions(self, content: str, file_path: str) -> List[Dict]:
        suggestions = []
        
        # Suggest structured logging
        if 'println!' in content and file_path.endswith('.rs'):
            suggestions.append({
                'title': 'Use structured logging',
                'description': 'Consider using a logging framework instead of println!',
                'code': 'use tracing::{info, warn, error};',
                'impact': 'Medium'
            })
        
        # Suggest proper error handling
        if '.unwrap()' in content and file_path.endswith('.rs'):
            suggestions.append({
                'title': 'Handle errors properly',
                'description': 'Consider using proper error handling instead of unwrap()',
                'code': '// Use .map_err() or ? operator instead',
                'impact': 'High'
            })
        
        # Suggest type annotations for Python
        if file_path.endswith('.py') and 'def ' in content:
            suggestions.append({
                'title': 'Add type hints',
                'description': 'Consider adding type annotations for better code clarity',
                'code': 'from typing import List, Dict, Optional',
                'impact': 'Medium'
            })
        
        return suggestions
    
    def calculate_score(self, content: str, issues: List[Dict]) -> float:
        score = 1.0
        lines = content.split('\n')
        total_lines = len(lines)
        
        if total_lines == 0:
            return 1.0
        
        # Deduct points for issues
        issue_penalty = len(issues) * 0.1
        score -= min(issue_penalty, 0.5)  # Cap at 50% penalty
        
        # Bonus for good practices
        if 'use tracing::' in content:
            score += 0.1
        if 'Result<' in content and str().endswith('.rs'):
            score += 0.1
        
        return max(0.0, min(1.0, score))
    
    def review_file(self, file_path: Path) -> CodeReview:
        try:
            with open(file_path, 'r', encoding='utf-8') as f:
                content = f.read()
            
            issues = self.analyze_code(content, str(file_path))
            suggestions = self.generate_suggestions(content, str(file_path))
            score = self.calculate_score(content, issues)
            
            return CodeReview(
                file_path=str(file_path),
                issues=issues,
                suggestions=suggestions,
                score=score
            )
        except Exception as e:
            print(f"Error reviewing {file_path}: {e}")
            return CodeReview(
                file_path=str(file_path),
                issues=[{'severity': 'Critical', 'message': f'Error reading file: {e}', 'line': None, 'code': None}],
                suggestions=[],
                score=0.0
            )
    
    def review_codebase(self) -> List[CodeReview]:
        print(f"Starting codebase review of: {self.args.path}")
        
        reviews = []
        path = Path(self.args.path)
        
        if not path.exists():
            print(f"Path {self.args.path} does not exist!")
            return reviews
        
        # Walk through all files
        for file_path in path.rglob('*'):
            if file_path.is_file() and self.is_code_file(file_path):
                print(f"Reviewing: {file_path}")
                review = self.review_file(file_path)
                reviews.append(review)
        
        print(f"Completed codebase review. Found {len(reviews)} files to review.")
        return reviews
    
    def save_reviews(self, reviews: List[CodeReview]) -> None:
        output_path = self.args.output or Path('code_review_results.json')
        
        # Convert reviews to JSON-serializable format
        reviews_data = []
        for review in reviews:
            reviews_data.append({
                'file_path': review.file_path,
                'issues': review.issues,
                'suggestions': review.suggestions,
                'score': review.score,
                'timestamp': review.timestamp.isoformat()
            })
        
        with open(output_path, 'w') as f:
            json.dump(reviews_data, f, indent=2)
        
        print(f"Review results saved to: {output_path}")
    
    def generate_patches(self, reviews: List[CodeReview]) -> None:
        print("Generating patches for suggested improvements...")
        
        for review in reviews:
            for suggestion in review.suggestions:
                if suggestion.get('code'):
                    # Create patch filename
                    safe_filename = review.file_path.replace('/', '_').replace('\\', '_')
                    safe_title = suggestion['title'].replace(' ', '_')
                    patch_name = f"{safe_filename}_{safe_title}.patch"
                    
                    # Create patch content
                    patch_content = f"""--- {review.file_path}
+++ {review.file_path}
@@ -1,1 +1,1 @@
{suggestion['code']}
"""
                    
                    with open(patch_name, 'w') as f:
                        f.write(patch_content)
                    
                    print(f"Generated patch: {patch_name}")
    
    def commit_changes(self) -> None:
        print("Committing changes to git...")
        
        try:
            # Git add
            result = subprocess.run(['git', 'add', '.'], capture_output=True, text=True)
            if result.returncode != 0:
                print("Git add failed")
                return
            
            # Git commit
            result = subprocess.run([
                'git', 'commit', '-m', 'Auto-generated code improvements from DevAgent'
            ], capture_output=True, text=True)
            
            if result.returncode == 0:
                print("Changes committed successfully")
            else:
                print("Git commit failed - no changes to commit")
                
        except Exception as e:
            print(f"Git operation failed: {e}")
    
    def run_interactive_mode(self) -> None:
        print("Starting interactive mode...")
        
        while True:
            print("\nDevAgent Interactive Mode")
            print("1. Review codebase")
            print("2. Generate patches")
            print("3. Commit changes")
            print("4. Exit")
            
            choice = input("Choose an option: ").strip()
            
            if choice == '1':
                reviews = self.review_codebase()
                self.save_reviews(reviews)
                print("Code review completed!")
            elif choice == '2':
                reviews = self.review_codebase()
                self.generate_patches(reviews)
                print("Patches generated!")
            elif choice == '3':
                self.commit_changes()
                print("Changes committed!")
            elif choice == '4':
                break
            else:
                print("Invalid option")

def main():
    parser = argparse.ArgumentParser(description='DevAgent Pipeline - AI-powered code review')
    parser.add_argument('--path', default='./src', help='Path to review')
    parser.add_argument('--output', help='Output file for results')
    parser.add_argument('--verbose', action='store_true', help='Enable verbose output')
    parser.add_argument('--interactive', action='store_true', help='Run in interactive mode')
    
    args = parser.parse_args()
    
    if args.verbose:
        print("DevAgent Pipeline v0.1.0 (Python)")
        print(f"Reviewing path: {args.path}")
    
    agent = DevAgent(args)
    
    if args.interactive:
        agent.run_interactive_mode()
    else:
        # Run automated review
        reviews = agent.review_codebase()
        
        # Save results
        agent.save_reviews(reviews)
        
        # Generate patches
        agent.generate_patches(reviews)
        
        # Optionally commit changes
        if reviews:
            agent.commit_changes()
        
        print("DevAgent pipeline completed successfully!")
        
        # Print summary
        total_issues = sum(len(r.issues) for r in reviews)
        total_suggestions = sum(len(r.suggestions) for r in reviews)
        avg_score = sum(r.score for r in reviews) / len(reviews) if reviews else 0
        
        print("\n=== Review Summary ===")
        print(f"Files reviewed: {len(reviews)}")
        print(f"Total issues found: {total_issues}")
        print(f"Total suggestions: {total_suggestions}")
        print(f"Average score: {avg_score:.2}")

if __name__ == '__main__':
    main() 