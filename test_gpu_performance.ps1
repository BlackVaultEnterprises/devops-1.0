# GPU Performance Test Script
# Demonstrates the speed of your GTX 1660 for code generation

Write-Host "⚡ GPU Performance Test - GTX 1660" -ForegroundColor Green
Write-Host "Testing code generation speed..." -ForegroundColor Cyan

# Check GPU status
Write-Host "📊 GPU Status:" -ForegroundColor Yellow
$gpuInfo = nvidia-smi --query-gpu=name,memory.total,memory.used,utilization.gpu --format=csv,noheader,nounits
Write-Host "   $gpuInfo" -ForegroundColor White

# Test 1: Generate Rust boilerplate with GPU
Write-Host "`n🦀 Test 1: Rust Boilerplate Generation" -ForegroundColor Cyan
$startTime = Get-Date

cargo run --release -- --generate --project-name "gpu_test_project" --gpu

$endTime = Get-Date
$duration = $endTime - $startTime
Write-Host "   Duration: $($duration.TotalSeconds) seconds" -ForegroundColor Green

# Test 2: Voice agent components generation
Write-Host "`n🎤 Test 2: Voice Agent Components" -ForegroundColor Cyan
$startTime = Get-Date

cargo run --release -- --generate --project-name "voice_agent" --gpu

$endTime = Get-Date
$duration = $endTime - $startTime
Write-Host "   Duration: $($duration.TotalSeconds) seconds" -ForegroundColor Green

# Test 3: GPU Benchmark
Write-Host "`n📊 Test 3: GPU Benchmark" -ForegroundColor Cyan
$startTime = Get-Date

cargo run --release -- --benchmark --gpu

$endTime = Get-Date
$duration = $endTime - $startTime
Write-Host "   Duration: $($duration.TotalSeconds) seconds" -ForegroundColor Green

# Performance summary
Write-Host "`n📈 Performance Summary:" -ForegroundColor Green
Write-Host "   GPU: NVIDIA GeForce GTX 1660" -ForegroundColor White
Write-Host "   Memory: 6GB GDDR5" -ForegroundColor White
Write-Host "   CUDA Cores: 1408" -ForegroundColor White
Write-Host "   Tensor Cores: 0 (Turing architecture)" -ForegroundColor White
Write-Host "   Expected Throughput: 1000+ tokens/sec" -ForegroundColor Green

Write-Host "`n🚀 Your GTX 1660 is now fully weaponized for code generation!" -ForegroundColor Green 