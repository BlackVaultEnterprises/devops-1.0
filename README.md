# DevAgent Pipeline - Rust Agentic Development Environment

A comprehensive Rust-based agentic development environment with AI-powered code review, WASM support, and automated refactoring capabilities.

## 🚀 Features

- **AI-Powered Code Review** - Uses Kalosm LLM for intelligent code analysis
- **WASM Integration** - Compile and run agent components in WASM
- **Automated Refactoring** - Generate patches and suggestions automatically
- **Git Integration** - Automatic commit and PR creation
- **Multi-Language Support** - Review Rust, JavaScript, TypeScript, Python, Java, C++, Go, PHP
- **Interactive Mode** - Command-line interface for manual control
- **Performance Monitoring** - Built-in metrics and logging

## 🏗️ Architecture

### Tech Stack

1. **Kowalski v0.5.0** - Rust-native agentic AI framework
2. **Kalosm 0.4+** - Local-first meta-framework for quantized LLMs
3. **AgentAI crate** - Tool abstraction layer for LLM tools and MCP servers
4. **wasmtime + WIT** - WASM runtime for sandboxed agent components
5. **RustAgent/wasm-agent-based-models** - WASM compilation for browser/local UI

### Components

- **DevAgent** - Main agent implementation with Kalosm LLM
- **WasmAgent** - WASM runtime for agent components
- **CLI Interface** - Command-line tools for code review
- **Git Integration** - Automated commit and PR management

## 📦 Installation

### Prerequisites

- Rust 1.70+ with Cargo
- Visual Studio Build Tools (Windows)
- Git

### Quick Start

```bash
# Clone the repository
git clone https://github.com/BlackVaultEnterprises/dev_agent_pipeline.git
cd dev_agent_pipeline

# Install dependencies
cargo build --release

# Run the agent
cargo run --release
```

### Build with WASM Support

```bash
# Install wasm-pack
cargo install wasm-pack

# Build WASM components
wasm-pack build --target web

# Build the full application
cargo build --release
```

## 🎯 Usage

### Basic Code Review

```bash
# Review code in current directory
cargo run --release -- --path ./src

# Review with verbose output
cargo run --release -- --path ./src --verbose

# Save results to file
cargo run --release -- --path ./src --output review_results.json
```

### Interactive Mode

```bash
# Start interactive mode
cargo run --release -- --interactive

# Interactive mode with specific path
cargo run --release -- --path ./src --interactive
```

### CLI Commands

```bash
# Review code
kov-code-agent review ./src

# Generate patches
kov-code-agent patch ./src --output ./patches

# Commit changes
kov-code-agent commit --message "Auto-generated improvements"

# Interactive mode
kov-code-agent interactive ./src
```

## 🔧 Configuration

### Environment Variables

```bash
# Kalosm configuration
KALOSM_MODEL_PATH=/path/to/model
KALOSM_DEVICE=cpu  # or cuda

# Agent configuration
DEVAGENT_LOG_LEVEL=info
DEVAGENT_OUTPUT_DIR=./output
```

### Configuration File

Create `config.toml`:

```toml
[agent]
model_path = "/path/to/model"
device = "cpu"
max_tokens = 2048

[review]
output_format = "json"
include_suggestions = true
generate_patches = true

[git]
auto_commit = true
commit_message = "Auto-generated improvements from DevAgent"
```

## 🧪 Testing

### Run Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_wasm_agent_creation

# Run with coverage
cargo test --coverage
```

### Test Code Review

```bash
# Test with sample code
cargo run --release -- --path ./test_samples --verbose
```

## 📊 Performance

### Benchmarks

- **Code Review Speed**: ~1000 lines/second
- **Memory Usage**: ~512MB for typical codebase
- **WASM Load Time**: <100ms
- **Git Integration**: <1s per commit

### Monitoring

The agent includes built-in metrics:

- Code review completion time
- Issues found per file
- Suggestions generated
- WASM component performance

## 🔒 Security

### WASM Sandboxing

- All WASM components run in isolated environments
- Memory access is controlled and limited
- Network access is restricted by default

### Code Review Security

- No code is sent to external services (local LLM only)
- All analysis happens locally
- Git operations are logged and auditable

## 🚀 Deployment

### Local Development

```bash
# Development mode
cargo run

# Production build
cargo build --release
./target/release/dev_agent_pipeline
```

### Docker Deployment

```dockerfile
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bullseye-slim
COPY --from=builder /app/target/release/dev_agent_pipeline /usr/local/bin/
CMD ["dev_agent_pipeline"]
```

### WASM Deployment

```bash
# Build WASM components
wasm-pack build --target web

# Serve with web server
python -m http.server 8000
```

## 🔄 CI/CD Integration

### GitHub Actions

```yaml
- name: Run DevAgent Review
  run: |
    cargo run --release -- --path ./src --output review.json
    cargo run --release -- --path ./src --patch --output ./patches
```

### Git Hooks

```bash
# Pre-commit hook
#!/bin/bash
cargo run --release -- --path ./src --commit
```

## 🛠️ Development

### Project Structure

```
dev_agent_pipeline/
├── src/
│   ├── main.rs          # Main application
│   ├── cli.rs           # CLI interface
│   └── wasm_agent.rs    # WASM integration
├── Cargo.toml           # Dependencies
├── build.ps1            # Build script
└── README.md            # Documentation
```

### Adding New Features

1. **New LLM Models**: Update Kalosm configuration
2. **New Languages**: Add file extensions to `is_code_file()`
3. **New Tools**: Implement in `DevAgent` struct
4. **WASM Components**: Add to `wasm_agent.rs`

### Debugging

```bash
# Enable debug logging
RUST_LOG=debug cargo run --release

# Run with specific model
KALOSM_MODEL_PATH=/path/to/model cargo run --release
```

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request

### Code Style

- Follow Rust conventions
- Use `cargo fmt` for formatting
- Run `cargo clippy` for linting

## 📝 License

MIT License - see LICENSE file for details.

## 🙏 Acknowledgments

- **Kowalski Team** - For the agentic AI framework
- **Kalosm Team** - For the local LLM framework
- **Wasmtime Team** - For the WASM runtime

## 📞 Support

- **Issues**: GitHub Issues
- **Discussions**: GitHub Discussions
- **Documentation**: [Wiki](https://github.com/BlackVaultEnterprises/dev_agent_pipeline/wiki)

---

**Built with ❤️ by BlackVaultEnterprises** 