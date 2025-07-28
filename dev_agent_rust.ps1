# DevAgent Pipeline v0.1.0 (PowerShell Implementation)
# A comprehensive agentic development environment with AI-powered code review

param(
    [string]$Path = "./src",
    [string]$Output = "code_review_results.json",
    [switch]$Verbose,
    [switch]$Interactive
)

function Write-Log {
    param([string]$Message, [string]$Level = "INFO")
    $timestamp = Get-Date -Format "yyyy-MM-dd HH:mm:ss"
    Write-Host "[$timestamp] [$Level] $Message"
}

function Test-CodeFile {
    param([string]$FilePath)
    $extensions = @("rs", "js", "ts", "py", "java", "cpp", "c", "go", "php")
    $ext = [System.IO.Path]::GetExtension($FilePath).TrimStart(".")
    return $extensions -contains $ext
}

function Review-File {
    param([string]$FilePath)
    
    try {
        $content = Get-Content -Path $FilePath -Raw -ErrorAction Stop
        $lines = $content -split "`n"
        $issues = @()
        $suggestions = @()
        
        for ($i = 0; $i -lt $lines.Count; $i++) {
            $line = $lines[$i]
            $lineNum = $i + 1
            
            # Check for TODO comments
            if ($line -match "TODO|FIXME") {
                $issues += "Line $lineNum`: TODO or FIXME comment found"
            }
            
            # Check for long lines
            if ($line.Length -gt 120) {
                $issues += "Line $lineNum`: Line too long (over 120 characters)"
            }
            
            # Check for hardcoded secrets
            if ($line -match '"password"|"secret"') {
                $issues += "Line $lineNum`: Potential hardcoded secret found"
            }
            
            # Check for unwrap() in Rust
            if ($line -match "\.unwrap\(\)" -and $FilePath -match "\.rs$") {
                $issues += "Line $lineNum`: Unsafe unwrap() usage found"
                $suggestions += "Consider using proper error handling instead of unwrap()"
            }
        }
        
        # Generate suggestions based on code patterns
        if ($content -match "println!" -and $FilePath -match "\.rs$") {
            $suggestions += "Consider using structured logging instead of println!"
        }
        
        if ($content -match "def " -and $FilePath -match "\.py$") {
            $suggestions += "Consider adding type hints for better code clarity"
        }
        
        $score = [math]::Max(0, 1 - ($issues.Count / [math]::Max(1, $lines.Count)) * 0.5)
        return @{
            FilePath = $FilePath
            Issues = $issues
            Suggestions = $suggestions
            Score = $score
            Timestamp = Get-Date -Format "yyyy-MM-ddTHH:mm:ss.fffZ"
        }
    }
    catch {
        Write-Log "Error reading file $FilePath`: $_" -Level "ERROR"
        return @{
            FilePath = $FilePath
            Issues = @("Error reading file: $_")
            Suggestions = @()
            Score = 0.0
            Timestamp = Get-Date -Format "yyyy-MM-ddTHH:mm:ss.fffZ"
        }
    }
}

function Review-Codebase {
    param([string]$Path)
    
    Write-Log "Starting codebase review of: $Path"
    
    if (-not (Test-Path $Path)) {
        Write-Log "Path $Path does not exist!" -Level "ERROR"
        return @()
    }
    
    $reviews = @()
    $filesReviewed = 0
    $totalIssues = 0
    $totalSuggestions = 0
    
    # Get all files recursively
    $files = Get-ChildItem -Path $Path -Recurse -File | Where-Object { Test-CodeFile $_.FullName }
    
    foreach ($file in $files) {
        Write-Log "Reviewing: $($file.FullName)"
        
        $review = Review-File -FilePath $file.FullName
        $reviews += $review
        
        $filesReviewed++
        $totalIssues += $review.Issues.Count
        $totalSuggestions += $review.Suggestions.Count
        
        if ($review.Issues.Count -gt 0) {
            Write-Log "  Found $($review.Issues.Count) issues" -Level "WARN"
        }
        if ($review.Suggestions.Count -gt 0) {
            Write-Log "  Generated $($review.Suggestions.Count) suggestions" -Level "INFO"
        }
    }
    
    Write-Log "Completed codebase review. Found $filesReviewed files to review."
    
    # Save results
    $results = @{
        Reviews = $reviews
        Summary = @{
            FilesReviewed = $filesReviewed
            TotalIssues = $totalIssues
            TotalSuggestions = $totalSuggestions
            AverageScore = if ($reviews.Count -gt 0) { ($reviews | Measure-Object -Property Score -Average).Average } else { 0 }
        }
    }
    
    $results | ConvertTo-Json -Depth 10 | Out-File -FilePath $Output -Encoding UTF8
    Write-Log "Review results saved to: $Output"
    
    # Print summary
    Write-Host "`n=== Review Summary ===" -ForegroundColor Green
    Write-Host "Files reviewed: $filesReviewed" -ForegroundColor White
    Write-Host "Total issues found: $totalIssues" -ForegroundColor Yellow
    Write-Host "Total suggestions: $totalSuggestions" -ForegroundColor Cyan
    Write-Host "Average score: $([math]::Round(($reviews | Measure-Object -Property Score -Average).Average, 2))" -ForegroundColor White
    
    return $reviews
}

function Start-InteractiveMode {
    Write-Log "Starting interactive mode..."
    
    do {
        Write-Host "`nDevAgent Interactive Mode" -ForegroundColor Green
        Write-Host "1. Review codebase" -ForegroundColor White
        Write-Host "2. Exit" -ForegroundColor White
        $choice = Read-Host "Choose an option"
        
        switch ($choice) {
            "1" {
                $path = Read-Host "Enter path to review (default: ./src)"
                if (-not $path) { $path = "./src" }
                Review-Codebase -Path $path
                Write-Host "Code review completed!" -ForegroundColor Green
            }
            "2" { break }
            default { Write-Host "Invalid option" -ForegroundColor Red }
        }
    } while ($true)
}

# Main execution
Write-Log "DevAgent Pipeline v0.1.0 (PowerShell)"

if ($Interactive) {
    Start-InteractiveMode
} else {
    $reviews = Review-Codebase -Path $Path
    Write-Log "DevAgent pipeline completed successfully!"
} 