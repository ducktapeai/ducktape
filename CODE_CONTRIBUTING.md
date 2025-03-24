# Contributing to DuckTape

First off, thank you for considering contributing to DuckTape! It's people like you that make DuckTape such a great tool.

## Code of Conduct

By participating in this project, you are expected to uphold our [Code of Conduct](CODE_OF_CONDUCT.md).

## Development Process

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes following our commit message conventions
4. Push to your branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

### Commit Message Format

We follow the Conventional Commits specification:

```
<type>(<scope>): <description>

[optional body]

[optional footer(s)]
```

Types:
- `feat`: A new feature
- `fix`: A bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, etc)
- `refactor`: Code changes that neither fix bugs nor add features
- `perf`: Performance improvements
- `test`: Adding or modifying tests
- `chore`: Changes to build process or auxiliary tools

Example:
```
feat(calendar): add support for recurring events

- Add weekly recurrence pattern
- Support end date and count options
- Add tests for recurrence logic

Closes #123
```

### Branch Naming Convention

- `feature/*`: New features
- `fix/*`: Bug fixes
- `docs/*`: Documentation changes
- `refactor/*`: Code refactoring
- `test/*`: Test additions or modifications

### Code Style

We follow Rust's official style guide and enforce it using `rustfmt`:

1. Run `cargo fmt` before committing
2. Follow the Rust API guidelines
3. Use meaningful variable names
4. Document public items with doc comments

### Testing

1. Write tests for new features
2. Ensure existing tests pass
3. Add integration tests when needed
4. Test edge cases

Run tests with:
```bash
cargo test
```

### Documentation

1. Update README.md if needed
2. Add doc comments to public items
3. Update API documentation
4. Include examples in doc comments

### Security

1. Run security checks:
```bash
./security-check.sh
```

2. Follow security best practices:
   - Validate user inputs
   - Handle errors properly
   - Use secure API key handling
   - Follow least privilege principle

### Performance

1. Use async/await appropriately
2. Minimize allocations
3. Profile code when needed
4. Use appropriate data structures

## Pull Request Process

1. Update documentation
2. Add/update tests
3. Run all checks:
   ```bash
   cargo fmt
   cargo clippy
   cargo test
   ./security-check.sh
   ```
4. Get reviews from maintainers
5. Address review comments
6. Update the PR as needed

### PR Description

Include:
- Clear description of changes
- Related issue numbers
- Breaking changes (if any)
- Testing performed
- Screenshots (if UI changes)

## Setting Up Development Environment

1. Install prerequisites:
   - Rust toolchain
   - Git
   - macOS (for Calendar integration)

2. Clone and setup:
   ```bash
   git clone https://github.com/YourUsername/ducktape.git
   cd ducktape
   cp .env.example .env
   # Add your API keys to .env
   ```

3. Install development tools:
   ```bash
   cargo install cargo-audit cargo-deny
   ```

4. Build the project:
   ```bash
   cargo build
   ```

## Getting Help

- Check existing issues
- Join our Discord community
- Read the documentation
- Ask in GitHub Discussions

## Recognition

Contributors will be:
- Listed in CONTRIBUTORS.md
- Mentioned in release notes
- Given credit in documentation

Thank you for contributing to DuckTape!