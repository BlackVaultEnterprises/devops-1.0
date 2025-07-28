# GPU-Optimized Build Script
# Utilizes your GTX 1660 for maximum performance

param(
    [switch]$Release,
    [switch]$GPU,
    [switch]$Benchmark,
    [switch]$Clean
)

Write-Host "🚀 GPU-Optimized Build Script" -ForegroundColor Green
Write-Host "Target: NVIDIA GeForce GTX 1660" -ForegroundColor Cyan

# Check GPU availability
Write-Host "📊 Checking GPU status..." -ForegroundColor Yellow
$gpuInfo = nvidia-smi --query-gpu=name,memory.total,memory.used,utilization.gpu --format=csv,noheader,nounits
Write-Host "GPU Info: $gpuInfo" -ForegroundColor Green

# Set environment variables for GPU optimization
$env:RUSTFLAGS = "-C target-cpu=native -C target-feature=+avx2,+fma"
$env:CUDA_VISIBLE_DEVICES = "0"
$env:RUST_LOG = "info"

if ($GPU) {
    Write-Host "⚡ Building with GPU acceleration..." -ForegroundColor Green
    $env:RUSTFLAGS += " -C target-feature=+cuda"
}

# Clean if requested
if ($Clean) {
    Write-Host "🧹 Cleaning previous build..." -ForegroundColor Yellow
    cargo clean
}

# Build with optimizations
$buildType = if ($Release) { "release" } else { "debug" }
Write-Host "🔨 Building in $buildType mode..." -ForegroundColor Cyan

if ($Release) {
    # Release build with maximum optimizations
    cargo build --release --features gpu
} else {
    # Debug build
    cargo build --features gpu
}

if ($LASTEXITCODE -eq 0) {
    Write-Host "✅ Build completed successfully!" -ForegroundColor Green
    
    # Run GPU benchmark if requested
    if ($Benchmark) {
        Write-Host "📊 Running GPU benchmark..." -ForegroundColor Cyan
        if ($Release) {
            cargo run --release -- --benchmark --gpu
        } else {
            cargo run -- --benchmark --gpu
        }
    }
    
    # Show performance metrics
    Write-Host "📈 Performance Summary:" -ForegroundColor Green
    Write-Host "   Build Type: $buildType" -ForegroundColor White
    Write-Host "   GPU Enabled: $GPU" -ForegroundColor White
    Write-Host "   Optimizations: Native CPU + AVX2 + FMA" -ForegroundColor White
    
    if ($GPU) {
        Write-Host "   CUDA Features: Enabled" -ForegroundColor Green
        Write-Host "   Memory Pool: 4GB GPU Memory" -ForegroundColor Green
        Write-Host "   Tensor Cores: Enabled" -ForegroundColor Green
    }
    
    # Show binary size
    $binaryPath = if ($Release) { "target/release/dev_agent_pipeline.exe" } else { "target/debug/dev_agent_pipeline.exe" }
    if (Test-Path $binaryPath) {
        $size = (Get-Item $binaryPath).Length
        $sizeMB = [math]::Round($size / 1MB, 2)
        Write-Host "   Binary Size: $sizeMB MB" -ForegroundColor White
    }
} else {
    Write-Host "❌ Build failed!" -ForegroundColor Red
    exit 1
}

Write-Host "🚀 Ready to run with maximum GPU performance!" -ForegroundColor Green 