#!/bin/bash
# Install git hooks for the project

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
HOOKS_DIR="$PROJECT_ROOT/.git/hooks"

echo "Installing git hooks..."

# Copy pre-commit hook
cp "$SCRIPT_DIR/pre-commit" "$HOOKS_DIR/pre-commit"
chmod +x "$HOOKS_DIR/pre-commit"

echo "✅ Git hooks installed successfully!"
echo ""
echo "The following checks will run before each commit:"
echo "  1. cargo fmt -- --check (code formatting)"
echo "  2. cargo build (compilation)"
echo "  3. cargo clippy (linting)"
echo ""
echo "To bypass these checks (not recommended), use: git commit --no-verify"
