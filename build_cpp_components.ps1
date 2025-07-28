# High-Performance C++ Component Builder
# This script builds whisper.cpp, llama.cpp, and piper with GPU acceleration

Write-Host "🚀 Building High-Performance C++ Components" -ForegroundColor Green

# Check if we're in the right directory
if (-not (Test-Path "Cargo.toml")) {
    Write-Error "Please run this script from the project root directory"
    exit 1
}

# Create components directory
$componentsDir = "components"
if (-not (Test-Path $componentsDir)) {
    New-Item -ItemType Directory -Path $componentsDir
}

Set-Location $componentsDir

# Function to check if command exists
function Test-Command($cmdname) {
    return [bool](Get-Command -Name $cmdname -ErrorAction SilentlyContinue)
}

# Check prerequisites
Write-Host "📋 Checking prerequisites..." -ForegroundColor Yellow

if (-not (Test-Command "git")) {
    Write-Error "Git is required but not found. Please install Git."
    exit 1
}

if (-not (Test-Command "cmake")) {
    Write-Error "CMake is required but not found. Please install CMake."
    exit 1
}

# Check for CUDA
$cudaAvailable = $false
if (Test-Command "nvcc") {
    $cudaAvailable = $true
    Write-Host "✅ CUDA detected - GPU acceleration will be enabled" -ForegroundColor Green
} else {
    Write-Host "⚠️  CUDA not detected - Building with CPU only" -ForegroundColor Yellow
}

# 1. Build whisper.cpp
Write-Host "🎤 Building whisper.cpp..." -ForegroundColor Cyan

if (-not (Test-Path "whisper.cpp")) {
    git clone https://github.com/ggerganov/whisper.cpp.git
}

Set-Location whisper.cpp

# Configure for GPU if available
if ($cudaAvailable) {
    Write-Host "Building whisper.cpp with CUDA support..." -ForegroundColor Green
    make -j
} else {
    Write-Host "Building whisper.cpp with CPU..." -ForegroundColor Yellow
    make -j
}

# Copy the binary
Copy-Item "main.exe" "../whisper.exe" -Force
Set-Location ..

# 2. Build llama.cpp
Write-Host "🧠 Building llama.cpp..." -ForegroundColor Cyan

if (-not (Test-Path "llama.cpp")) {
    git clone https://github.com/ggerganov/llama.cpp.git
}

Set-Location llama.cpp

# Configure for GPU if available
if ($cudaAvailable) {
    Write-Host "Building llama.cpp with CUDA support..." -ForegroundColor Green
    make -j LLAMA_CUBLAS=1
} else {
    Write-Host "Building llama.cpp with CPU..." -ForegroundColor Yellow
    make -j
}

# Copy the binary
Copy-Item "main.exe" "../llama.exe" -Force
Set-Location ..

# 3. Build Piper TTS
Write-Host "🔊 Building Piper TTS..." -ForegroundColor Cyan

if (-not (Test-Path "piper")) {
    git clone https://github.com/rhasspy/piper.git
}

Set-Location piper

# Create build directory
if (-not (Test-Path "build")) {
    New-Item -ItemType Directory -Path "build"
}

Set-Location build

# Configure with CMake
if ($cudaAvailable) {
    Write-Host "Configuring Piper with CUDA support..." -ForegroundColor Green
    cmake .. -DUSE_CUDA=ON
} else {
    Write-Host "Configuring Piper with CPU..." -ForegroundColor Yellow
    cmake ..
}

# Build
Write-Host "Building Piper..." -ForegroundColor Green
cmake --build . --config Release -j

# Copy the binary
Copy-Item "src/piper.exe" "../../piper.exe" -Force
Set-Location ../..

# 4. Download models
Write-Host "📥 Downloading models..." -ForegroundColor Cyan

$modelsDir = "models"
if (-not (Test-Path $modelsDir)) {
    New-Item -ItemType Directory -Path $modelsDir
}

Set-Location $modelsDir

# Download Whisper model
if (-not (Test-Path "ggml-base.bin")) {
    Write-Host "Downloading Whisper base model..." -ForegroundColor Yellow
    Invoke-WebRequest -Uri "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin" -OutFile "ggml-base.bin"
}

# Download Llama model (TinyLlama for testing)
if (-not (Test-Path "TinyLlama-1.1B-Chat-v1.0.Q4_K_M.gguf")) {
    Write-Host "Downloading TinyLlama model..." -ForegroundColor Yellow
    Invoke-WebRequest -Uri "https://huggingface.co/TheBloke/TinyLlama-1.1B-Chat-v1.0-GGUF/resolve/main/TinyLlama-1.1B-Chat-v1.0.Q4_K_M.gguf" -OutFile "TinyLlama-1.1B-Chat-v1.0.Q4_K_M.gguf"
}

# Download Piper voice model
if (-not (Test-Path "en_US-amy-low.onnx")) {
    Write-Host "Downloading Piper voice model..." -ForegroundColor Yellow
    Invoke-WebRequest -Uri "https://huggingface.co/rhasspy/piper-voices/resolve/main/en/en_US/amy/low/en_US-amy-low.onnx" -OutFile "en_US-amy-low.onnx"
}

Set-Location ..

# 5. Create configuration
Write-Host "⚙️  Creating configuration..." -ForegroundColor Cyan

$config = @{
    whisper_path = "components/whisper.exe"
    llama_path = "components/llama.exe"
    piper_path = "components/piper.exe"
    model_path = "components/models/TinyLlama-1.1B-Chat-v1.0.Q4_K_M.gguf"
    voice_model_path = "components/models/en_US-amy-low.onnx"
    qdrant_url = "http://localhost:6333"
    indradb_url = "http://localhost:8080"
    gpu_enabled = $cudaAvailable
    max_concurrent_requests = 10
}

$config | ConvertTo-Json -Depth 10 | Out-File "orchestrator_config.json" -Encoding UTF8

# 6. Create Docker Compose for databases
Write-Host "🐳 Creating Docker Compose for databases..." -ForegroundColor Cyan

$dockerCompose = @"
version: '3.8'

services:
  qdrant:
    image: qdrant/qdrant:latest
    ports:
      - "6333:6333"
    volumes:
      - qdrant_data:/qdrant/storage
    environment:
      - QDRANT__SERVICE__HTTP_PORT=6333

  indradb:
    image: indradb/indradb:latest
    ports:
      - "8080:8080"
    volumes:
      - indradb_data:/data

volumes:
  qdrant_data:
  indradb_data:
"@

$dockerCompose | Out-File "docker-compose.yml" -Encoding UTF8

# 7. Create startup script
Write-Host "🚀 Creating startup script..." -ForegroundColor Cyan

$startupScript = @"
# High-Performance Voice Agent Startup Script

Write-Host "🎤 Starting High-Performance Voice Agent" -ForegroundColor Green

# Start databases
Write-Host "📊 Starting databases..." -ForegroundColor Yellow
docker-compose up -d

# Wait for databases to be ready
Write-Host "⏳ Waiting for databases..." -ForegroundColor Yellow
Start-Sleep -Seconds 10

# Start the Rust orchestrator
Write-Host "🚀 Starting Rust orchestrator..." -ForegroundColor Green
cargo run --release -- --voice --gpu

Write-Host "✅ Voice agent is running!" -ForegroundColor Green
Write-Host "🎤 Speak to interact with your agent" -ForegroundColor Cyan
"@

$startupScript | Out-File "start_voice_agent.ps1" -Encoding UTF8

Write-Host "✅ Build completed successfully!" -ForegroundColor Green
Write-Host "📁 Components built in: $componentsDir" -ForegroundColor Cyan
Write-Host "🚀 Run: .\start_voice_agent.ps1" -ForegroundColor Green

Set-Location .. 