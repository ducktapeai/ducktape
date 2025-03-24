# DuckTape Development Guide

This guide explains how to set up your development environment and contribute to DuckTape.

## Architecture Overview

DuckTape is structured into several key components:

### Core Components
- `calendar.rs`: Calendar integration and event management
- `command_parser.rs`: Natural language command parsing
- `api_server.rs`: REST API endpoints
- `websocket_server.rs`: WebSocket server for real-time updates
- `zoom.rs`: Zoom meeting integration

### State Management
- `state.rs`: Application state management
- `config.rs`: Configuration handling
- `env_manager.rs`: Environment variable management

### Security
- `security.rs`: Security utilities and encryption
- `api_keys.rs`: API key management
- `validation.rs`: Input validation

## Development Environment Setup

### Prerequisites
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install development tools
cargo install cargo-audit cargo-deny cargo-watch cargo-edit cargo-outdated
```

### Local Setup
1. Clone the repository:
```bash
git clone https://github.com/DuckTapeAI/ducktape.git
cd ducktape
```

2. Copy environment template:
```bash
cp .env.example .env
```

3. Install pre-commit hooks:
```bash
pre-commit install
```

### Docker Development
```bash
# Start all services
docker-compose up -d

# Run tests
docker-compose run test

# Run benchmarks
docker-compose run benchmark

# Run security checks
docker-compose run security-check
```

## Development Workflow

### Building
```bash
# Development build
cargo build

# Release build
cargo build --release
```

### Testing
```bash
# Run all tests
cargo test --all-features

# Run specific test
cargo test test_name

# Run integration tests
./run-integration-tests.sh
```

### Code Quality
```bash
# Format code
cargo fmt

# Run linter
cargo clippy -- -D warnings

# Run security audit
./security-check.sh
```

### Hot Reloading
```bash
cargo watch -x run
```

## Project Structure

### Source Code Organization
- `src/`: Main source code
  - `bin/`: Binary executables
  - `commands/`: Command implementations
  - `services/`: Service layer implementations

### Test Organization
- `tests/`: Integration tests
- `benches/`: Performance benchmarks

### Configuration
- `Cargo.toml`: Project dependencies
- `rustfmt.toml`: Code formatting rules
- `deny.toml`: Dependency policies
- `.github/`: GitHub Actions and templates

## Code Style Guidelines

### Rust Conventions
- Follow Rust API guidelines
- Use meaningful variable names
- Document public items
- Handle errors appropriately

### Documentation
- Add doc comments to public items
- Include examples in documentation
- Keep README.md updated
- Document breaking changes

### Testing
- Write unit tests for new features
- Add integration tests for APIs
- Test edge cases
- Include performance tests for critical paths

## Common Tasks

### Adding a New Command
1. Create command handler in `src/commands/`
2. Add parser rules in `command_parser.rs`
3. Register command in `command_processor.rs`
4. Add tests in `tests/`

### Implementing Calendar Features
1. Add calendar operation in `calendar.rs`
2. Implement WebSocket notifications if needed
3. Add validation in `validation.rs`
4. Update API documentation

### Security Considerations
1. Validate all inputs
2. Use proper error handling
3. Implement rate limiting
4. Follow least privilege principle
5. Run security checks

## Debugging

### Logging
```rust
debug!("Processing command: {}", cmd);
error!("Failed to connect: {}", err);
```

### Error Handling
```rust
use anyhow::{Context, Result};
fn process() -> Result<()> {
    operation().context("Failed to process")?;
    Ok(())
}
```

### Performance Profiling
```bash
# Run benchmarks
./run-benchmarks.sh

# Generate flamegraph
cargo flamegraph
```

## Release Process

1. Update version in `Cargo.toml`
2. Update CHANGELOG.md
3. Run full test suite
4. Create release PR
5. Tag release after merge

## Getting Help

- Check existing issues
- Join our Discord
- Review documentation
- Ask in GitHub Discussions

## Best Practices

### Security
- Never commit API keys
- Use secure API key handling
- Implement proper validation
- Follow security guidelines

### Performance
- Profile before optimizing
- Use async appropriately
- Minimize allocations
- Cache effectively

### Code Quality
- Write clear documentation
- Add comprehensive tests
- Handle errors properly
- Follow Rust idioms