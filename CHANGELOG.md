# Changelog

All notable changes to the Ducktape project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.16.22] - 2025-05-12
### Changed
- Refactored utility functions for improved clarity and maintainability
- Enhanced calendar extraction in NLP commands
- Improved time parsing for relative dates

## [0.16.21] - 2025-05-11
### Fixed
- Improved time extraction for 'tonight' pattern in natural language parser.

## [0.16.20] - 2025-05-07
### Fixed
- Fixed email address formatting issue in calendar invitations where domain names could be duplicated
- Improved contact email validation to prevent malformed addresses
- Enhanced email deduplication logic to properly handle similar addresses
- Added additional logging for email address processing to improve debugging

## [0.16.19] - 2025-05-07
### Added
- Added support for explicit time range extraction in natural language commands (e.g., "from 8pm to 9pm")

### Fixed
- Fixed issue where specifying time ranges like "from 9pm to 10pm" would result in the event being scheduled for 9pm-10pm instead of 8pm-9pm
- Improved time extraction patterns to accurately handle explicit time ranges

## [0.16.18] - 2025-05-07
### Added
- Enhanced special case handling for common time expressions in natural language processing
- Improved command suffix extraction for calendar operations

### Fixed
- Fixed time extraction issues with specific patterns like "tonight at 7pm" and "tomorrow at 9am"
- Fixed command suffix handling for options like "--zoom" and contact lists
- Improved time pattern detection accuracy in the natural language parser
- Enhanced 12-hour to 24-hour time conversion logic

## [0.16.17] - 2025-05-07
### Added
- Enhanced contact lookup functionality with user-friendly messages
- Improved automatic launch of Contacts app when not running
- Added better feedback during contact resolution process

### Changed
- Refined error handling for contact lookup failures
- Updated documentation for Homebrew release process
- Standardized user feedback messages in terminal output

### Fixed
- Fixed issue with contact lookup not providing user feedback
- Removed unnecessary parentheses in conditional statements following Rust style guidelines
- Improved contact email collection and deduplication logic

## [0.16.16] - 2025-05-07
### Added
- Improved command verb mapping for natural language processing
- Enhanced detection of meeting-related keywords

### Changed
- Refactored natural language processing pipeline for better maintainability
- Improved command normalization to prevent duplicate prefix issues

### Fixed
- Fixed issue with "create an zoom meeting" command not properly recognized
- Enhanced command structure validation for natural language inputs
- Improved integration between command mapping and time extraction

## [0.16.15] - 2025-05-06
### Added
- Enhanced support for recurring event patterns in calendar operations
- Improved handling of timezone conversions

### Changed
- Refined error messaging for better user experience
- Updated dependencies for better performance

### Fixed
- Fixed edge cases in time parsing for specific date formats
- Improved stability in WebSocket connections

## [0.16.14] - 2025-05-05
### Added
- Enhanced time pattern detection in natural language processing

### Changed
- Updated dependency versions for security improvements
- Improved error handling for API requests

### Fixed
- Fixed "tomorrow morning at X" time pattern in natural language processing
- Cleaned up CHANGELOG structure for better maintainability

## [0.16.13] - 2025-05-05
### Changed
- Enhanced code organization following Rust coding standards
- Improved module structure for better maintainability
- Updated dependencies for security and performance

### Fixed
- Fixed parser module imports for consistent API
- Improved error handling in state management
- Enhanced validation for user inputs
- Fixed "tomorrow morning at X" time pattern in natural language processing

## [0.16.12] - 2025-05-04
### Changed
- Version bump for new release
- Minor stability improvements
- Updated dependencies

### Fixed
- Fixed time parsing in natural language commands with specific time expressions (e.g., "tonight at 7pm")
- Added proper AM/PM to 24-hour time conversion for calendar events
- Improved time extraction from user input to ensure correct event scheduling
- Added comprehensive documentation and test cases for time-related parsing

## [0.16.10] - 2025-04-25
### Fixed
- Fixed time parsing in natural language event titles (e.g., "tonight at 7pm")
- Added proper extraction of time expressions from event titles
- Implemented comprehensive time pattern detection for calendar events

## [0.16.9] - 2025-04-25
### Added
- Enhanced contact extraction functionality for calendar events
- Support for "and invite" pattern in natural language commands

### Fixed
- Fixed compilation issues in parser modules
- Improved error handling in command processing
- Fixed type safety issues in parser trait implementations

## [0.16.8] - 2025-04-25
### Changed
- Version bump

## [0.16.7] - 2025-04-25
### Changed
- Version bump

## [0.16.6] - 2025-04-25
### Changed
- Added contacts to llm model processing

## [0.16.2] - 2025-04-25
### Changed
- Improved stability and performance for calendar integrations
- Enhanced error handling for network requests
- Updated dependencies for better security

## [0.16.1] - 2025-04-26
### Fixed
- Fixed natural language processing for "create an event" commands
- Improved command parsing for event creation without explicit calendar prefix
- Enhanced sanitization of natural language inputs in the Grok parser

## [0.16.0] - 2025-04-25
### Added
- Enhanced natural language processing for calendar events
- Improved command detection for event creation phrases
- Added automated prefix handling for natural language commands

### Fixed
- Fixed issue with "create an event" commands not being properly recognized
- Fixed natural language parsing for event creation without explicit prefixes
- Added comprehensive test cases for natural language event creation

## [0.15.5] - 2025-04-25
### Added
- New release version 0.15.5

### Changed
- Updated version information across project

### Fixed
- Minor bug fixes and stability improvements

## [0.15.4] - 2025-04-25
### Added
- New release version

### Changed
- Updated version information across project

### Fixed
- Minor bug fixes and improvements

## [0.15.0] - 2025-04-25
### Changed
- Completely removed OpenAI parser and all dependencies
- Simplified LLM provider options to only Grok and DeepSeek
- Updated code structure for more maintainable architecture
- Improved error handling for better diagnostics

### Fixed
- Fixed numerous code quality issues flagged by Clippy
- Improved use of idiomatic Rust patterns throughout codebase
- Fixed unnecessary parentheses in conditional statements
- Added proper Default implementations for key structs

## [0.14.1] - 2025-04-25
### Fixed
- Fixed GrokParser implementation to properly use X.AI API key
- Removed fallback to OpenAI API in Grok parser module
- Improved error handling for missing API keys

## [0.14.0] - 2025-04-25
### Changed
- Refactored parser modules for cleaner structure
- Removed unused DeepSeek modules to streamline codebase
- Fixed duplicate module declarations in lib.rs

## [0.13.6] - 2025-04-23
### Fixed
- Fixed parser bugs and improved error handling

## [0.13.5] - 2025-04-23
### Fixed
- Refactored the parser

## [0.13.4] - 2025-04-22
### Fixed
- Fixed note creation commands with multi-word titles and arguments
- Improved CLI argument parsing to properly handle space-separated parameters
- Enhanced input validation for note commands

## [0.13.3] - 2025-04-22
### Fixed
- Fixed note creation commands with multi-word titles and arguments
- Improved CLI argument parsing to properly handle space-separated parameters
- Enhanced input validation for note commands

## [0.13.2] - 2025-04-21
### Fixed
- Fixed calendar event creation bug where end times incorrectly included dates
- Added post-processing step to ensure proper time format in generated commands
- Improved natural language parsing for calendar events

## [0.13.1] - 2025-04-21
### Fixed
- Restored validation.rs file to fix build issues in main branch
- Fixed dependency inconsistency between branches

## [0.13.0] - 2025-04-20
### Changed
- Major refactoring of calendar module into separate components
- Consolidated calendar functionality into specialized files
- Enhanced email validation in calendar operations
- Improved contact lookup logic for calendar events
- Restructured calendar code for better maintainability

## [0.12.1] - 2025-04-16
### Added
- Added exit command handler to terminate the application properly
- Exit commands now work in both Terminal Mode and Natural Language Mode
- Added "exit" to help documentation for better discoverability

### Changed
- Modified app.rs to detect exit/quit commands early in the processing pipeline
- Improved terminal experience by ensuring exit commands work even if LLM API calls fail

## [0.11.20] - 2025-04-15
### Changed
- Started new development cycle with minor version bump
- Preparing for new feature additions

## [0.11.19] - 2025-04-15
### Changed
- Started new development cycle with minor version bump
- Preparing for new feature additions

## [0.11.0] - 2025-03-10
### Added
- Initial stable release with core functionality
- Basic calendar management
- Contact management
- Command-line interface

## [0.10.0] - 2025-02-15
### Added
- Beta release with major features implemented
- Testing framework established

## [0.9.0] - 2025-01-20
### Added
- Alpha release for testing
- Core architecture established

## [0.1.0] - 2024-11-01
### Added
- Project initialized
- Basic framework established
