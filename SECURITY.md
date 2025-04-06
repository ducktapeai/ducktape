# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.10.x  | :white_check_mark: |
| 0.1.x   | :x:               |

## Reporting a Vulnerability

We take the security of DuckTape seriously. If you believe you have found a security vulnerability, please report it to us as described below.

**Please do not report security vulnerabilities through public GitHub issues.**

Instead, please report them via email to security@ducktape.ai (once established). If possible, encrypt your message with our PGP key (to be published).

You should receive a response within 48 hours. If for some reason you do not, please follow up via email to ensure we received your original message.

Please include the following information in your report:

- Type of issue (e.g. buffer overflow, SQL injection, cross-site scripting, etc.)
- Full paths of source file(s) related to the manifestation of the issue
- The location of the affected source code (tag/branch/commit or direct URL)
- Any special configuration required to reproduce the issue
- Step-by-step instructions to reproduce the issue
- Proof-of-concept or exploit code (if possible)
- Impact of the issue, including how an attacker might exploit it

## Security Practices

DuckTape implements several security measures:

1. **API Key Protection**
   - All API keys are stored securely using environment variables
   - Keys are never logged or exposed in error messages
   - The `secrecy` crate is used for secure credential handling

2. **Input Validation**
   - All user inputs are validated and sanitized
   - AppleScript commands are strictly validated to prevent injection attacks
   - Regular expressions are pre-compiled for efficiency

3. **Secure Dependencies**
   - Regular security audits using `cargo audit`
   - Dependency version constraints are maintained
   - Policy enforcement using `cargo-deny`

4. **Error Handling**
   - Proper error handling to prevent information leakage
   - No unwrap() on production code paths
   - Custom error types with appropriate detail levels

5. **File System Security**
   - Path traversal prevention
   - Proper file permissions handling
   - Secure temporary file management

6. **Network Security**
   - HTTPS for all external API calls
   - WebSocket connections are validated
   - Rate limiting on API endpoints

## Secure Development

We follow these practices for secure development:

1. **Code Review**
   - All changes must go through code review
   - Security-sensitive changes require additional review
   - Automated CI/CD security checks

2. **Testing**
   - Security-focused test cases
   - Regular penetration testing
   - Fuzzing of input handlers

3. **Monitoring**
   - Error tracking and monitoring
   - Regular security assessments
   - Dependency vulnerability tracking

## Secure Installation

When installing DuckTape:

1. Always download from trusted sources (crates.io or official GitHub releases)
2. Verify checksums of downloaded artifacts
3. Keep your Rust toolchain updated
4. Follow the principle of least privilege when setting up API keys

## Security Updates

Security updates will be released as soon as possible after a vulnerability is confirmed. Updates will be published:

1. As new versions on crates.io
2. As GitHub releases
3. With notifications in our security advisories

## Attribution

We are committed to working with security researchers and respecting their efforts. We will credit researchers who report security issues (unless they prefer to remain anonymous).