# Contributing to DuckTape

Thank you for considering contributing to DuckTape! This document provides guidelines and instructions for contributing.

## Code of Conduct

Please be respectful and considerate of others when contributing to this project.

## Getting Started

1. Fork the repository on GitHub
2. Clone your fork locally
3. Set up the development environment as described in the README
4. Create a branch for your changes

## Development Setup

1. Ensure you have Rust installed
2. Clone the repository
3. Install dependencies with `cargo build`
4. Run the security check script: `./security-check.sh`

## Making Changes

1. Create a topic branch from the main branch
2. Make your changes following the code style of the project
3. Add or update tests as necessary
4. Run the test suite to ensure your changes don't break existing functionality
5. Make sure your code passes all lints with `cargo clippy`

## Pull Requests

1. Update your fork to include the latest changes from the main repository
2. Submit a pull request targeting the main branch
3. Clearly describe the problem and solution in the PR description
4. Include any relevant issue numbers in the PR description

## Coding Standards

- Follow Rust standard naming conventions
- Document your code with comments and docstrings
- Write unit tests for new functionality
- Ensure all tests pass
- Use the security-check.sh script to validate security aspects

## Security Considerations

- Validate all user inputs to prevent injection attacks
- Avoid using `unwrap()` and `expect()` in production code
- Handle errors appropriately
- Be mindful of permissions when accessing files and executing commands

Thank you for your contributions!
