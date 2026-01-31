#!/bin/bash
# Automated release script for declarch
set -e

VERSION=$1
if [ -z "$VERSION" ]; then
    echo "Usage: $0 X.Y.Z"
    echo "Example: $0 0.5.3"
    exit 1
fi

# Validate version format
if [[ ! $VERSION =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    echo "Error: Invalid version format. Use X.Y.Z (e.g., 0.5.3)"
    exit 1
fi

echo "🔄 Preparing release $VERSION..."

# Check for uncommitted changes
if [ -n "$(git status --porcelain)" ]; then
    echo "⚠️  Warning: You have uncommitted changes"
    git status --short
    echo "Commit them first before releasing"
    exit 1
fi

# Update Cargo.toml
echo "📝 Updating Cargo.toml..."
sed -i "s/^version = \".*\"/version = \"$VERSION\"/" Cargo.toml

# Update install.sh
echo "📝 Updating install.sh..."
sed -i "s/^VERSION=\".*\"/VERSION=\"$VERSION\"/" install.sh

# Run checks
echo "🔍 Running tests..."
cargo test --all --quiet

echo "🔍 Running clippy..."
cargo clippy --all-targets --all-features -- -D warnings

echo "🔍 Checking formatting..."
cargo fmt -- --check

# Commit changes
echo "💾 Committing changes..."
git add Cargo.toml install.sh
git commit -m "chore: prepare release $VERSION"

echo "📊 Summary of changes:"
git show --stat HEAD

echo ""
echo "🏷️  About to create and push tag v$VERSION..."
echo "Press Ctrl+C to cancel, or Enter to continue"
read

# Create and push tag
git tag "v$VERSION"
git push
git push origin "v$VERSION"

echo ""
echo "✅ Release $VERSION prepared!"
echo ""
echo "Next steps:"
echo "  1. Check CI: https://github.com/nixval/declarch/actions"
echo "  2. Wait for build to complete (~5 minutes)"
echo "  3. Verify release: https://github.com/nixval/declarch/releases/tag/v$VERSION"
echo "  4. Test install: curl -fsSL https://raw.githubusercontent.com/nixval/declarch/main/install.sh | sh"
echo ""
echo "To verify the binary after release:"
echo "  wget https://github.com/nixval/declarch/releases/download/v$VERSION/declarch-x86_64-unknown-linux-gnu.tar.gz"
echo "  tar xzf declarch-x86_64-unknown-linux-gnu.tar.gz"
echo "  ./declarch --version  # Should show $VERSION"
