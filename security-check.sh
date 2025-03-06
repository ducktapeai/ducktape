#!/bin/bash
# Security check script for DuckTape project

echo "🔒 Running DuckTape security checks..."

# Check if cargo-audit is installed, install if not
if ! command -v cargo-audit &> /dev/null; then
    echo "Installing cargo-audit..."
    cargo install cargo-audit
fi

# Check if cargo-deny is installed, install if not
if ! command -v cargo-deny &> /dev/null; then
    echo "Installing cargo-deny..."
    cargo install cargo-deny
fi

echo
echo "🧪 Running cargo audit..."
cargo audit
AUDIT_EXIT=$?

echo
echo "📦 Running cargo deny check (licenses, sources, vulnerabilities)..."
cargo deny check
DENY_EXIT=$?

echo
echo "🔍 Running Clippy with security lints..."
cargo clippy -- -W clippy::all -W clippy::pedantic -W clippy::nursery -W clippy::cargo -W clippy::unwrap_used -W clippy::expect_used 
CLIPPY_EXIT=$?

echo
echo "📝 Checking for vulnerable patterns..."
echo "Checking for unwrap() calls..."
grep -r "unwrap()" --include="*.rs" src/
UNWRAP_COUNT=$?

echo "Checking for expect() calls..."
grep -r "expect(" --include="*.rs" src/
EXPECT_COUNT=$?

echo "Checking for panic!() calls..."
grep -r "panic!" --include="*.rs" src/
PANIC_COUNT=$?

echo "Checking for unsafe blocks..."
grep -r "unsafe" --include="*.rs" src/
UNSAFE_COUNT=$?

echo "Checking for potential command injection vulnerabilities..."
grep -r "Command::new" --include="*.rs" src/ | grep -v "escape_"
COMMAND_COUNT=$?

echo "Checking for temporary file security issues..."
grep -r "tempfile" --include="*.rs" src/ 
TEMPFILE_COUNT=$?

echo "Checking for sensitive data exposure..."
grep -r "API_KEY\|SECRET\|PASSWORD" --include="*.rs" src/ | grep -v "env::var"
SENSITIVE_COUNT=$?

echo "Checking for proper URL parsing in HTTP requests..."
grep -r "http://" --include="*.rs" src/ 
HTTP_COUNT=$?

echo
echo "📊 Security Check Summary:"
if [ $AUDIT_EXIT -eq 0 ]; then
    echo "✅ Cargo audit: No known vulnerabilities found"
else
    echo "❌ Cargo audit: Vulnerabilities detected"
fi

if [ $DENY_EXIT -eq 0 ]; then
    echo "✅ Cargo deny: No issues found"
else
    echo "❌ Cargo deny: Issues detected"
fi

if [ $CLIPPY_EXIT -eq 0 ]; then
    echo "✅ Clippy: No linting issues found"
else
    echo "❌ Clippy: Linting issues detected"
fi

if [[ $UNWRAP_COUNT -ne 0 && $EXPECT_COUNT -ne 0 && $PANIC_COUNT -ne 0 && $UNSAFE_COUNT -ne 0 ]]; then
    echo "✅ No basic vulnerable patterns found"
else
    echo "⚠️  Potentially risky unwrap/expect/panic/unsafe patterns detected - review output above"
fi

if [ $COMMAND_COUNT -ne 0 ]; then
    echo "⚠️  Potential command injection risks detected - ensure all user inputs are properly escaped"
fi

if [ $TEMPFILE_COUNT -eq 0 ]; then
    echo "✅ No temporary file usage detected"
else
    echo "⚠️  Temporary file usage detected - ensure proper security controls are in place"
fi

if [ $SENSITIVE_COUNT -ne 0 ]; then
    echo "⚠️  Potential sensitive data exposure detected - review how secrets are handled"
else
    echo "✅ No hardcoded secrets found"
fi

if [ $HTTP_COUNT -ne 0 ]; then
    echo "⚠️  Unencrypted HTTP connections detected - consider using HTTPS for all connections"
else
    echo "✅ No unencrypted HTTP connections found"
fi

echo
echo "🛡️ Security recommendations:"
echo "1. Regularly run 'cargo audit' to check for vulnerabilities in dependencies"
echo "2. Use Result<T, E> instead of unwrap()/expect() for proper error handling"
echo "3. Avoid unsafe blocks where possible"
echo "4. Always validate and sanitize user inputs before using them in commands"
echo "5. Use cargo-deny to enforce dependency security policies"
echo "6. Consider adding integration tests focused on security boundaries"

# Make executable
chmod +x security-check.sh