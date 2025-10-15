#!/bin/bash
# Install git hooks for TSLM development

set -e

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
HOOKS_DIR="$REPO_ROOT/.git/hooks"

echo "Installing git hooks for TSLM..."

# Create pre-commit hook
cat > "$HOOKS_DIR/pre-commit" << 'EOF'
#!/bin/bash
# Pre-commit hook to run cargo fmt check and clippy

set -e

echo "Running pre-commit checks..."

# Check formatting
echo "Checking code formatting..."
if ! cargo fmt --all -- --check; then
    echo "❌ Formatting check failed!"
    echo "Run 'cargo fmt --all' to fix formatting issues."
    exit 1
fi
echo "✅ Formatting check passed"

# Run clippy
echo "Running clippy..."
if ! cargo clippy --all-targets --all-features --workspace -- -D warnings; then
    echo "❌ Clippy check failed!"
    echo "Fix all clippy warnings before committing."
    exit 1
fi
echo "✅ Clippy check passed"

echo "✅ All pre-commit checks passed!"
exit 0
EOF

# Make hook executable
chmod +x "$HOOKS_DIR/pre-commit"

echo "✅ Git hooks installed successfully!"
echo ""
echo "The following hooks are now active:"
echo "  - pre-commit: Runs cargo fmt and clippy checks"
echo ""
echo "To bypass hooks (not recommended), use: git commit --no-verify"
