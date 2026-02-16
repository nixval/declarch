#!/bin/bash
# Automated release script for declarch
set -e

sed_in_place() {
    local expr="$1"
    local file="$2"
    if sed --version >/dev/null 2>&1; then
        sed -i "$expr" "$file"
    else
        sed -i '' "$expr" "$file"
    fi
}

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

if [ ! -f "RELEASE_CHECKLIST.md" ]; then
    echo "Error: RELEASE_CHECKLIST.md not found"
    exit 1
fi

# Check for uncommitted changes
if [ -n "$(git status --porcelain)" ]; then
    echo "⚠️  Warning: You have uncommitted changes"
    git status --short
    echo "Commit them first before releasing"
    exit 1
fi

# Update Cargo.toml
echo "📝 Updating Cargo.toml..."
sed_in_place "s/^version = \".*\"/version = \"$VERSION\"/" Cargo.toml

# Update installer scripts
echo "📝 Updating installers..."
sed_in_place "s/^VERSION=\".*\"/VERSION=\"$VERSION\"/" install.sh
sed_in_place "s/\\[string\\]\\$Version = \".*\"/[string]\$Version = \"$VERSION\"/" install.ps1

# Run checks
echo "🔍 Running tests..."
cargo test --all-targets --quiet

echo "🔍 Running clippy..."
cargo clippy --all-targets -- -D warnings

echo "🔍 Checking formatting..."
cargo fmt --check

# Changelog discipline checks
echo "📝 Verifying changelog..."
if ! grep -q '^## \[Unreleased\]' CHANGELOG.md; then
    echo "Error: CHANGELOG.md must contain an [Unreleased] section"
    exit 1
fi

if ! awk '
  /^## \[Unreleased\]/ { in_unreleased=1; next }
  /^## \[/ && in_unreleased { in_unreleased=0 }
  in_unreleased && /^- / { found=1 }
  END { exit found ? 0 : 1 }
' CHANGELOG.md; then
    echo "Error: Add at least one bullet under CHANGELOG.md [Unreleased]"
    exit 1
fi

# Commit changes
echo "💾 Committing changes..."
git add Cargo.toml install.sh install.ps1
git commit -m "chore: prepare release $VERSION"

echo "📊 Summary of changes:"
git show --stat HEAD

echo ""
echo "Checklist reference: RELEASE_CHECKLIST.md"
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
echo "  4. Test install (Linux/macOS): curl -fsSL https://raw.githubusercontent.com/nixval/declarch/main/install.sh | sh"
echo "  5. Test install (Windows): irm https://raw.githubusercontent.com/nixval/declarch/main/install.ps1 | iex"
echo ""
echo "To verify the binary after release:"
echo "  wget https://github.com/nixval/declarch/releases/download/v$VERSION/declarch-x86_64-unknown-linux-gnu.tar.gz"
echo "  tar xzf declarch-x86_64-unknown-linux-gnu.tar.gz"
echo "  ./declarch --version  # Should show $VERSION"
