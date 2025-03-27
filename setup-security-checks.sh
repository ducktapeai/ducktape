#!/usr/bin/env bash
#
# Setup security checks for Ducktape project
# This script installs pre-commit hooks for credential scanning
# and ensures code meets project standards before commits
#
# Following Ducktape Rust project standards for security

set -e

# Display banner
echo "🦆 Setting up Ducktape security checks..."

# Create .git/hooks directory if it doesn't exist
mkdir -p .git/hooks

# Install pre-commit hook
echo "📥 Installing pre-commit hook for credential scanning..."
cp -f hooks/pre-commit .git/hooks/
chmod +x .git/hooks/pre-commit
echo "✅ Pre-commit hook installed successfully."

# Create default patterns file if it doesn't exist
if [ ! -f "sensitive-patterns.txt" ]; then
    echo "📝 Creating default sensitive patterns file..."
    cat > sensitive-patterns.txt << 'EOF'
# Sensitive patterns for Ducktape project
# Add additional patterns here to detect in pre-commit hook

# API keys and tokens
api_key
access_key
client_secret
token
auth_token
service_account

# Specific services
zoom_client_id
zoom_client_secret
zoom_account_id
openai_api_key
xai_api_key

# Credentials
password
secret
private_key
EOF
    echo "✅ Created sensitive-patterns.txt with default patterns."
fi

# Ensure gitleaks config exists in project
mkdir -p .github
if [ ! -f ".github/gitleaks.toml" ]; then
    echo "📝 Creating Gitleaks configuration..."
    cp -n hooks/gitleaks.toml .github/gitleaks.toml 2>/dev/null || echo "⚠️ .github/gitleaks.toml already exists."
fi

# Create Github workflow if it doesn't exist 
if [ ! -f ".github/workflows/credential-scan.yml" ]; then
    echo "📝 Creating GitHub Actions workflow for credential scanning..."
    mkdir -p .github/workflows
    cp -n hooks/credential-scan.yml .github/workflows/credential-scan.yml 2>/dev/null || echo "⚠️ Workflow file already exists."
fi

echo ""
echo "🔒 Security checks setup complete!"
echo "Pre-commit hooks will now scan for credentials before each commit."
echo "You can bypass these checks if needed with: git commit --no-verify"
echo ""
echo "ℹ️ For GitHub Actions:"
echo "1. Go to repository Settings → Billing and plans → Spending limit"
echo "2. Set the spending limit to $0.00 to stay within free tier limits"
echo "3. This ensures your credential scanning won't incur unexpected costs"