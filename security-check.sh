#!/bin/bash
# Security check script for DuckTape project

echo "🔒 Running DuckTape security checks..."
echo

echo "🧪 Running cargo audit..."
cargo audit
AUDIT_EXIT=$?

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

echo
echo "📊 Security Check Summary:"
if [ $AUDIT_EXIT -eq 0 ]; then
    echo "✅ Cargo audit: No known vulnerabilities found"
else
    echo "❌ Cargo audit: Vulnerabilities detected"
fi

if [ $CLIPPY_EXIT -eq 0 ]; then
    echo "✅ Clippy: No linting issues found"
else
    echo "❌ Clippy: Linting issues detected"
fi

if [[ $UNWRAP_COUNT -ne 0 && $EXPECT_COUNT -ne 0 && $PANIC_COUNT -ne 0 && $UNSAFE_COUNT -ne 0 ]]; then
    echo "✅ No vulnerable patterns found"
else
    echo "⚠️  Potentially risky patterns detected - review output above"
fi

# Make executable
chmod +x security-check.sh