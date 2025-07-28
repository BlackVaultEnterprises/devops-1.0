# 🎤 High-Performance Voice Agent System

A **compiled, multi-process architecture** orchestrated by Rust that eliminates interpreter overhead and enables true parallel processing of conversation and memory with **GPU acceleration**.

## 🏗️ Architecture Overview

```mermaid
graph TD
    subgraph "Foreground Process: Operator Agent (Rust)"
        A[🎤 STT: whisper.cpp] --> B{Orchestrator};
        B -- Text --> C[🧠 LLM: llama.cpp];
        C -- Response --> B;
        B -- Text --> D[🔊 TTS: Piper + RVC];
        B -- Async Query --> E{🚀 High-Speed IPC Bus<br/>(gRPC / Apache Arrow)};
    end

    subgraph "Background Process: Memory Agent (Rust)"
        F[🧠 Memory Controller] <--> E;
        F <--> G[Tier 1: In-RAM Cache];
        F <--> H[Tier 2: Qdrant Vector DB];
        F <--> I[Tier 3: IndraDB Knowledge Graph];
    end
```

## 🚀 Key Features

- **🎤 Voice Cloning**: Use your 3 WAV files to create a personalized voice
- **🧠 Local Brain**: Phi-3-mini-instruct for local decision making
- **⚡ GPU Acceleration**: Full CUDA support for whisper.cpp, llama.cpp, and Piper
- **💾 Infinite Storage**: WASM-based storage with Qdrant vector database
- **🔗 MCP Integration**: Cloud delegation via Model Context Protocol
- **🎯 Hands-Free Operation**: Complete voice-controlled development environment

## 📋 Prerequisites

### Hardware Requirements
- **GPU**: NVIDIA GeForce GTX 1660 or better (6GB+ VRAM)
- **RAM**: 16GB+ recommended
- **Storage**: 50GB+ for models and voice data

### Software Requirements
- **Windows 10/11** with latest updates
- **Visual Studio Build Tools 2019+**
- **CUDA Toolkit 12.7+** (for GPU acceleration)
- **Docker Desktop** (for databases)
- **Git** and **CMake**

## 🛠️ Installation

### Step 1: Build C++ Components

```powershell
# Run the build script to compile whisper.cpp, llama.cpp, and Piper
.\build_cpp_components.ps1
```

This script will:
- ✅ Clone and build whisper.cpp with GPU support
- ✅ Clone and build llama.cpp with CUDA acceleration
- ✅ Clone and build Piper TTS with GPU support
- ✅ Download required models (Whisper, TinyLlama, Piper voice)
- ✅ Create Docker Compose for databases
- ✅ Generate configuration files

### Step 2: Clone Your Voice

```powershell
# Use your 3 WAV files to create a voice clone
.\clone_voice.ps1 -VoiceName "YourName" -AudioFiles @("voice1.wav", "voice2.wav", "voice3.wav") -GPU
```

This will:
- ✅ Copy your WAV files to the voice models directory
- ✅ Create training scripts for your voice
- ✅ Set up GPU-accelerated voice training
- ✅ Generate synthesis scripts

### Step 3: Train Your Voice

```powershell
# Navigate to your voice directory
cd voice_models\YourName

# Train your voice model (GPU accelerated)
.\train_voice.ps1
```

### Step 4: Start the Voice Agent

```powershell
# Start databases and launch the voice agent
.\start_voice_agent.ps1
```

## 🎯 Usage

### Voice Commands

The system supports various voice commands:

**Local Commands** (processed by Phi-3-mini-instruct):
- "Open file main.rs"
- "Build the project"
- "Run tests"
- "Save all files"
- "Create new function"

**Cloud Commands** (delegated to cloud LLMs):
- "Analyze this codebase"
- "Generate documentation"
- "Optimize performance"
- "Research best practices"

### Hands-Free Development

```bash
# Start the agent with voice control
cargo run --release -- --voice --gpu

# Speak naturally:
"Open the main.rs file"
"Build the project with release mode"
"Run the tests and show me the results"
"Create a new function called process_data"
```

## 🔧 Configuration

### Orchestrator Configuration

The system uses `orchestrator_config.json`:

```json
{
  "whisper_path": "components/whisper.exe",
  "llama_path": "components/llama.exe", 
  "piper_path": "components/piper.exe",
  "model_path": "components/models/TinyLlama-1.1B-Chat-v1.0.Q4_K_M.gguf",
  "voice_model_path": "voice_models/YourName/models/voice_model.pth",
  "qdrant_url": "http://localhost:6333",
  "indradb_url": "http://localhost:8080",
  "gpu_enabled": true,
  "max_concurrent_requests": 10
}
```

### Voice Configuration

Each voice clone has its own configuration in `voice_models/YourName/voice_config.json`:

```json
{
  "name": "YourName",
  "audio_files": ["voice1.wav", "voice2.wav", "voice3.wav"],
  "created_at": "2024-01-15 10:30:00",
  "gpu_enabled": true,
  "sample_rate": 16000,
  "channels": 1
}
```

## 🏗️ Architecture Details

### Multi-Process Design

1. **Operator Agent (Rust)**: Main orchestrator
   - Manages STT, LLM, and TTS subprocesses
   - Handles high-speed IPC communication
   - Coordinates voice command processing

2. **Memory Agent (Rust)**: Background memory management
   - Tier 1: In-RAM cache for instant access
   - Tier 2: Qdrant vector database for semantic search
   - Tier 3: IndraDB knowledge graph for relationships

### GPU Acceleration

- **whisper.cpp**: CUDA-accelerated speech-to-text
- **llama.cpp**: GPU-accelerated LLM inference with LLAMA_CUBLAS
- **Piper TTS**: GPU-accelerated text-to-speech synthesis
- **Voice Training**: PyTorch with CUDA for voice model training

### High-Speed IPC

- **gRPC**: For inter-process communication
- **Apache Arrow**: For efficient data serialization
- **Tokio channels**: For async message passing
- **Shared memory**: For audio data transfer

## 🔍 Performance Metrics

### Expected Performance (GTX 1660)

| Component | CPU Mode | GPU Mode | Improvement |
|-----------|----------|----------|-------------|
| STT (whisper.cpp) | 2x real-time | 0.5x real-time | 4x faster |
| LLM (llama.cpp) | 5 tokens/sec | 25 tokens/sec | 5x faster |
| TTS (Piper) | 1x real-time | 0.3x real-time | 3x faster |
| Voice Training | 1 epoch/hour | 1 epoch/10min | 6x faster |

### Memory Usage

- **RAM**: ~8GB during operation
- **VRAM**: ~4GB with GPU acceleration
- **Storage**: ~20GB for models and voice data

## 🛠️ Troubleshooting

### Common Issues

**1. CUDA not detected**
```powershell
# Install CUDA Toolkit
choco install cuda --version=12.7.0
```

**2. Build tools missing**
```powershell
# Install Visual Studio Build Tools
choco install visualstudio2019buildtools
```

**3. Docker not running**
```powershell
# Start Docker Desktop
Start-Process "C:\Program Files\Docker\Docker\Docker Desktop.exe"
```

**4. Voice training fails**
```powershell
# Install Python dependencies
pip install torch torchaudio transformers datasets accelerate
```

### Debug Mode

```powershell
# Run with verbose logging
cargo run --release -- --voice --gpu --verbose
```

## 🔮 Future Enhancements

- **🎭 Emotion Control**: Voice synthesis with emotional inflection
- **🌍 Multi-Language**: Support for multiple languages
- **🔗 Plugin System**: Extensible MCP server integration
- **📊 Analytics**: Voice command analytics and optimization
- **🎨 Custom UI**: Web-based voice agent dashboard

## 📚 API Reference

### Voice Agent API

```rust
// Initialize voice agent
let voice_agent = VoiceAgent::new(config).await?;

// Clone voice from audio files
let voice_id = voice_agent.clone_voice(audio_files, "YourName").await?;

// Synthesize speech
let response = voice_agent.synthesize_speech(request).await?;

// Start voice listener
voice_agent.start_voice_listener().await?;
```

### Local Brain API

```rust
// Initialize local brain
let brain = LocalBrain::new(config).await?;

// Process voice command
let response = brain.process_voice_command(command).await?;

// Execute action
brain.execute_action(&response).await?;
```

### Orchestrator API

```rust
// Initialize orchestrator
let orchestrator = Orchestrator::new(config).await?;

// Process audio
let stt_result = orchestrator.process_audio(audio_chunk).await?;

// Generate response
let llm_response = orchestrator.generate_response(request).await?;

// Synthesize speech
let tts_response = orchestrator.synthesize_speech(request).await?;
```

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Implement your changes
4. Add tests and documentation
5. Submit a pull request

## 📄 License

This project is licensed under the MIT License - see the LICENSE file for details.

## 🙏 Acknowledgments

- **whisper.cpp**: High-performance speech recognition
- **llama.cpp**: Efficient LLM inference
- **Piper**: Fast text-to-speech synthesis
- **Qdrant**: Vector database for semantic search
- **IndraDB**: Knowledge graph database
- **Rust**: For high-performance orchestration

---

**🎤 Ready to build your hands-free development environment? Start with the installation guide above!** 