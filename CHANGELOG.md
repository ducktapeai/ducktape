# Changelog

All notable changes to DuckTape will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.2] - 2025-03-29

### Changed
- Updated dependencies to latest compatible versions
- Improved error handling in WebSocket connections
- Enhanced security in API key management

### Fixed
- Resolved dependency conflicts in Cargo.lock
- Fixed error handling in calendar event creation

## [0.1.1] - 2025-03-29

### Fixed
- Resolved all compiler warnings across the codebase
- Fixed missing BufRead trait import in env_debug.rs
- Added Serialize derive for ScheduleCommand in command_parser.rs
- Added #[allow(dead_code)] attributes to properly document future-use code
- Fixed unused imports across multiple modules
- Improved code organization according to Rust coding standards

### Changed
- Enhanced code quality by fixing all compiler warnings
- Made appropriate uses of #[allow(dead_code)] for code stability
- Improved code readability and maintainability

## [0.1.0] - 2025-03-24

### Added
- Initial open source release
- Natural language command processing using OpenAI, Grok, or DeepSeek
- Apple Calendar integration
  - Event creation and management
  - Recurring events support
  - Calendar selection
  - Event search
- WebSocket API server
  - Real-time command processing
  - Client authentication
  - Rate limiting
  - Secure communication
- Zoom meeting integration
  - Automatic meeting creation
  - Meeting link insertion
- Contact group management
- Environment variable management
- Security features
  - Input validation
  - Safe API key handling
  - WebSocket security
- Comprehensive documentation
  - API documentation
  - Development guide
  - Security policy

### Security
- Added security audit script
- Implemented input validation
- Added rate limiting
- Added TLS support for WebSocket server
- Added secure API key handling

### Changed
- Updated to use dotenvy instead of dotenv
- Improved error handling throughout
- Enhanced WebSocket protocol
- Optimized calendar operations

### Fixed
- Calendar event creation edge cases
- WebSocket connection handling
- Environment variable loading
- Input validation issues

## [0.0.1] - 2025-03-15

### Added
- Initial development version
- Basic calendar integration
- Simple command parsing
- Environment configuration
