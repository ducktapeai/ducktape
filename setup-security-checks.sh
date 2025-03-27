#!/bin/bash
#
# DuckTape Security Setup Script
# This script sets up pre-commit hooks and other security measures for the DuckTape project

set -e

echo "🦆 DuckTape Security Setup"
echo "=========================="

# Install pre-commit hook
if [ -d .git ]; then
    echo "📋 Installing pre-commit hook for credential scanning..."
    
    if [ -f hooks/pre-commit ]; then
        cp hooks/pre-commit .git/hooks/
        chmod +x .git/hooks/pre-commit
        echo "✅ Pre-commit hook installed successfully"
    else
        echo "❌ Error: hooks/pre-commit not found"
        exit 1
    fi
else
    echo "❌ Error: Not a git repository"
    exit 1
fi

# Check for sensitive-patterns.txt
if [ ! -f sensitive-patterns.txt ]; then
    echo "⚠️ Warning: sensitive-patterns.txt not found"
    echo "Creating default sensitive-patterns.txt file..."
    
    cat > sensitive-patterns.txt << 'EOL'
# DuckTape Credential Scanning Patterns
# Format: pattern==>replacement or regex:pattern==>replacement

# API Keys by service
xai-[a-zA-Z0-9]{20,}==>REMOVED-XAI-KEY
sk-[a-zA-Z0-9]{32,}==>REMOVED-OPENAI-KEY

# Generic credential patterns
regex:(?i)(api_key|secret|token|password|key)[\s]*[=:][\s]*["][^\s"]{8,}["](?!.*example)==>REMOVED-CREDENTIAL
regex:(?i)(zoom[_-]?(client[_-]?id|client[_-]?secret|account[_-]?id))[=:]["']?[\w\-]{16,}["']?==>REMOVED-ZOOM-CREDENTIAL
EOL

    echo "✅ Created sensitive-patterns.txt with default patterns"
fi

# Provide instructions for CI setup
echo ""
echo "🚀 CI Pipeline Setup"
echo "-------------------"
echo "A GitHub Actions workflow has been set up in:"
echo ".github/workflows/credential-scan.yml"
echo ""
echo "This will scan for credentials on every push and pull request."
echo ""
echo "👉 Security Recommendations:"
echo "1. Always use environment variables instead of hardcoded credentials"
echo "2. Keep your .env file in .gitignore (already set up)"
echo "3. Use 'git commit --no-verify' only when absolutely necessary"
echo "4. Regularly rotate your API keys and secrets"
echo "5. Run 'cargo audit' regularly to check for dependency vulnerabilities"
echo ""
echo "✅ Security setup complete"

# Set up .env.example if it doesn't exist
if [ ! -f .env.example ]; then
    echo ""
    echo "📄 Creating .env.example file..."
    
    cat > .env.example << 'EOL'
# DuckTape Environment Variables Example
# Copy this file to .env and fill in your values

# API Keys for Language Models (choose at least one)
OPENAI_API_KEY=your_openai_api_key_here
XAI_API_KEY=your_xai_api_key_here
DEEPSEEK_API_KEY=your_deepseek_api_key_here

# Zoom Integration (optional)
ZOOM_ACCOUNT_ID=your_zoom_account_id
ZOOM_CLIENT_ID=your_zoom_client_id
ZOOM_CLIENT_SECRET=your_zoom_client_secret

# Optional Configuration
# DUCKTAPE_LOG_LEVEL=info
# DUCKTAPE_CONFIG_PATH=/custom/path/to/config
EOL

    echo "✅ Created .env.example file"
fi