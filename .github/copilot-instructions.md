# Rust Coding Standards for Ducktape Project

## Version Control and Branching

1. **Branch Management**
   - Always create a new branch before making changes (never work directly on main)
   - Use descriptive branch names with prefixes: `feature/`, `bugfix/`, `docs/`, `refactor/`
   - Keep branches focused on a single issue or feature
   - Rebase feature branches on main before submitting PR
   - Delete branches after merging

## Code Style and Formatting

1. **Code Formatting**
   - Use `rustfmt` with the project's `rustfmt.toml` configuration
   - Run `cargo fmt` before committing changes
   - Maintain consistent 4-space indentation
   - Maximum line length of 100 characters

2. **Naming Conventions**
   - Use `snake_case` for functions, variables, and modules
   - Use `PascalCase` for types, traits, and enums
   - Use `SCREAMING_SNAKE_CASE` for constants and static variables
   - Prefix unsafe functions with `unsafe_`
   - Use descriptive names that reflect purpose

3. **Documentation**
   - Document all public APIs with rustdoc comments
   - Include examples in documentation when functionality isn't immediately obvious
   - Use `//` for single-line comments and `///` for documentation comments
   - Document error conditions and panics
   - Keep documentation up to date with code changes

## Code Organization

1. **Module Structure**
   - One module per file
   - Group related functionality into modules
   - Use `pub(crate)` for items only needed within the crate
   - Keep modules focused and single-purpose
   - Place tests in a `tests` submodule with `#[cfg(test)]`

2. **Dependencies**
   - Minimize external dependencies
   - Use `thiserror` for error handling
   - Prefer async/await with Tokio for asynchronous code
   - Keep dependencies up to date using `cargo update`
   - Document reason for each dependency in `Cargo.toml`

## Error Handling

1. **Error Types**
   - Use custom error types with `thiserror`
   - Implement detailed error messages
   - Return `Result` types for fallible operations
   - Use `anyhow::Result` for functions where specific error types aren't needed
   - Avoid unwrap() except in tests or when failure is impossible

2. **Logging**
   - Use the `log` crate for logging
   - Include appropriate log levels (error, warn, info, debug)
   - Log important state changes and error conditions
   - Include relevant context in log messages
   - Avoid logging sensitive information

## Performance and Safety

1. **Memory Management**
   - Minimize cloning where possible
   - Use references instead of owned values when appropriate
   - Implement Drop trait when managing resources
   - Use Arc/Mutex for thread-safe shared state
   - Avoid unsafe code unless absolutely necessary

2. **Concurrency**
   - Use Tokio for async operations
   - Prefer async/await over direct thread manipulation
   - Use proper synchronization primitives
   - Avoid blocking operations in async contexts
   - Document thread safety assumptions

## Testing

1. **Unit Tests**
   - Write tests for all public functions
   - Use descriptive test names (test_what_when_then)
   - Mock external dependencies
   - Test error conditions
   - Maintain test coverage

2. **Integration Tests**
   - Write integration tests for key functionality
   - Test realistic usage scenarios
   - Use test fixtures when appropriate
   - Test API boundaries
   - Include performance-critical test cases

## Version Control

1. **Commits**
   - Write clear commit messages
   - Keep commits focused and atomic
   - Reference issue numbers when applicable
   - Run tests before committing
   - Update documentation with code changes

## Security

1. **Security Practices**
   - Follow Rust security best practices
   - Validate all input data
   - Use constant-time comparisons for sensitive data
   - Keep dependencies updated for security patches
   - Review security implications of unsafe code

## Performance Considerations

1. **Optimization**
   - Profile before optimizing
   - Document performance-critical sections
   - Use appropriate data structures
   - Consider memory usage
   - Benchmark critical paths

## Code Review

1. **Review Checklist**
   - Verify rustfmt compliance
   - Check error handling
   - Review test coverage
   - Validate documentation
   - Consider performance implications

## Tooling

1. **Required Tools**
   - rustfmt for formatting
   - clippy for linting
   - cargo-audit for security
   - cargo-deny for dependency checking
   - cargo-watch for development