# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.3] - 2024-06-10

### Added
- Proper CLI version command (`ducktape --version`) for displaying version information
- Improved help command with detailed descriptions (`ducktape --help`)
- Better command-line argument handling with more robust parsing
- Natural language processing improvements using Grok parser
- Enhanced error logging throughout the application

### Fixed
- Resolved dependency conflicts and updated all packages to latest versions
- Fixed issue with cargo dependency resolution in Cargo.lock
- Corrected API server initialization errors when starting in terminal mode
- Improved error handling in command processing pipeline

### Changed
- Refactored command execution process for better extensibility
- Updated development dependencies to improve build process
- Improved documentation and code comments across the project

## [0.1.2] - 2024-05-20

### Added
- Initial public release
- Basic calendar management functionality
- Natural language processing for calendar events
- Terminal UI for interactive usage
- WebSocket API server for desktop client integration

## [0.1.1] - 2024-05-01

### Added
- Core application foundation
- Command pattern implementation
- Basic CLI infrastructure

## [0.1.0] - 2024-04-15

### Added
- Project initialization

[0.1.3]: https://github.com/ducktapeai/ducktape/compare/v0.1.2...v0.1.3
[0.1.2]: https://github.com/ducktapeai/ducktape/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/ducktapeai/ducktape/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/ducktapeai/ducktape/releases/tag/v0.1.0
