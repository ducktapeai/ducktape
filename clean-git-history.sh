#!/bin/bash

# Ensure we have a clean working directory
git stash

# Update .gitignore
echo ".git.bak" >> .gitignore
echo ".git.bak/**/*" >> .gitignore
echo "target/" >> .gitignore
echo "target/**/*" >> .gitignore

# Make sure the changes to .gitignore are committed
git add .gitignore
git commit -m "Update .gitignore to exclude large files"

# Use git-filter-repo to remove large files from history
git filter-repo --path-glob 'target/*' --path-glob '.git.bak/*' --invert-paths --force

# Clean up
git gc --aggressive --prune=now

# Show remaining large files
echo "Checking remaining large files..."
git rev-list --objects --all | git cat-file --batch-check='%(objecttype) %(objectname) %(objectsize) %(rest)' | sort -nr -k3 | head -n 5

# Force push to remote (uncomment when ready)
# git push origin --force --all
