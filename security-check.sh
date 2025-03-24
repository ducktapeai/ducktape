#!/bin/bash
set -e

echo "🔒 Running DuckTape Security Checks"

# Check for known vulnerabilities in dependencies
echo "Checking dependencies for vulnerabilities..."
cargo audit

# Run cargo-deny checks
echo "Running license and advisory checks..."
cargo deny check licenses
cargo deny check bans
cargo deny check sources
cargo deny check advisories

# Check for sensitive data in code
echo "Checking for sensitive data..."
sensitive_patterns=(
    "api[_]?key"
    "auth[_]?token"
    "password"
    "secret"
    "private[_]?key"
    "[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}"
)

for pattern in "${sensitive_patterns[@]}"; do
    echo "Checking for pattern: $pattern"
    matches=$(git grep -I -l -E "$pattern" -- '*.rs' '*.toml' '*.json' '*.yml' '*.yaml' || true)
    if [ ! -z "$matches" ]; then
        for file in $matches; do
            if [ "$file" != "src/api_keys.rs" ] && \
               [ "$file" != "src/security.rs" ] && \
               [ "$file" != ".env.example" ]; then
                echo "⚠️  Warning: Potential sensitive data in $file"
            fi
        done
    fi
done

# Check for unsafe code usage
echo "Checking for unsafe code blocks..."
unsafe_matches=$(git grep -l "unsafe" -- '*.rs' || true)
if [ ! -z "$unsafe_matches" ]; then
    echo "⚠️  Warning: Unsafe code blocks found in:"
    echo "$unsafe_matches"
fi

# Run Clippy with additional security lints
echo "Running security-focused Clippy checks..."
cargo clippy -- \
    -W clippy::all \
    -W clippy::pedantic \
    -W clippy::nursery \
    -W clippy::cargo \
    -D clippy::unwrap_used \
    -D clippy::expect_used \
    -D clippy::panic \
    -D clippy::integer-arithmetic \
    -D clippy::float-arithmetic

# Check for outdated dependencies
echo "Checking for outdated dependencies..."
cargo outdated

# Verify file permissions
echo "Checking file permissions..."
find . -type f -name "*.rs" -perm /111 -exec chmod 644 {} \;

# Check for large files
echo "Checking for large files..."
find . -type f -size +10M -not -path "./target/*" -exec ls -lh {} \;

# Check for proper error handling
echo "Checking error handling patterns..."
results=$(git grep -l "unwrap()" -- '*.rs' || true)
if [ ! -z "$results" ]; then
    echo "⚠️  Warning: Found unwrap() calls that should be handled properly:"
    echo "$results"
fi

# Verify all API endpoints use proper validation
echo "Checking API endpoint validation..."
api_files=$(find . -type f -name "*.rs" -exec grep -l "pub async fn" {} \;)
for file in $api_files; do
    if ! grep -q "validate" "$file"; then
        echo "⚠️  Warning: API endpoint in $file might be missing validation"
    fi
done

# Check for proper CORS configuration
echo "Checking CORS configuration..."
if grep -r "Access-Control-Allow-Origin: \*" .; then
    echo "⚠️  Warning: Found potentially unsafe CORS configuration"
fi

# Check WebSocket security
echo "Checking WebSocket security..."
ws_files=$(find . -type f -name "*.rs" -exec grep -l "WebSocket" {} \;)
for file in $ws_files; do
    if ! grep -q "Origin" "$file"; then
        echo "⚠️  Warning: WebSocket in $file might be missing origin checks"
    fi
done

# Verify proper TLS configuration
echo "Checking TLS configuration..."
tls_files=$(find . -type f -name "*.rs" -exec grep -l "TlsConnector" {} \;)
for file in $tls_files; do
    if ! grep -q "min_protocol_version" "$file"; then
        echo "⚠️  Warning: TLS configuration in $file might not enforce minimum version"
    fi
done

echo "✅ Security checks completed!"