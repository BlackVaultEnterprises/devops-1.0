# DevAgent Pipeline Build Script
# Builds the Rust agent with WASM support

Write-Host "Building DevAgent Pipeline..." -ForegroundColor Green

# Check if Rust is installed
if (-not (Get-Command rustc -ErrorAction SilentlyContinue)) {
    Write-Host "Rust not found. Please install Rust first." -ForegroundColor Red
    exit 1
}

# Check if wasm-pack is installed
if (-not (Get-Command wasm-pack -ErrorAction SilentlyContinue)) {
    Write-Host "Installing wasm-pack..." -ForegroundColor Yellow
    cargo install wasm-pack
}

# Build the main application
Write-Host "Building main application..." -ForegroundColor Yellow
cargo build --release

if ($LASTEXITCODE -ne 0) {
    Write-Host "Build failed!" -ForegroundColor Red
    exit 1
}

# Build WASM components
Write-Host "Building WASM components..." -ForegroundColor Yellow
wasm-pack build --target web

if ($LASTEXITCODE -ne 0) {
    Write-Host "WASM build failed!" -ForegroundColor Red
    exit 1
}

# Create distribution directory
$distDir = "dist"
if (-not (Test-Path $distDir)) {
    New-Item -ItemType Directory -Path $distDir
}

# Copy built binaries
Write-Host "Copying built binaries..." -ForegroundColor Yellow
Copy-Item "target/release/dev_agent_pipeline.exe" $distDir -ErrorAction SilentlyContinue
Copy-Item "target/release/dev_agent_pipeline" $distDir -ErrorAction SilentlyContinue

# Copy WASM files
Copy-Item "pkg/*" $distDir -Recurse -ErrorAction SilentlyContinue

# Create launcher script
$launcherScript = @"
#!/bin/bash
# DevAgent Pipeline Launcher

echo "Starting DevAgent Pipeline..."
./dev_agent_pipeline `$@
"@

$launcherScript | Out-File -FilePath "$distDir/run.sh" -Encoding UTF8

# Create Windows batch file
$batchFile = @"
@echo off
echo Starting DevAgent Pipeline...
dev_agent_pipeline.exe %*
"@

$batchFile | Out-File -FilePath "$distDir/run.bat" -Encoding ASCII

Write-Host "Build completed successfully!" -ForegroundColor Green
Write-Host "Distribution files created in: $distDir" -ForegroundColor Cyan 