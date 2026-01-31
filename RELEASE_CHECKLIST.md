# Release Checklist

Use this checklist when preparing a new release to avoid common issues.

## Pre-Release

1. **Update version numbers**
   ```bash
   # Update Cargo.toml
   vim Cargo.toml  # Change version = "X.Y.Z"

   # Update install.sh
   vim install.sh  # Change VERSION="X.Y.Z"

   # Update CHANGELOG.md
   # Add new section for X.Y.Z with all changes
   ```

2. **Run tests**
   ```bash
   cargo test --all
   cargo clippy --all-targets --all-features -- -D warnings
   cargo fmt -- --check
   ```

3. **Commit changes**
   ```bash
   git add -A
   git commit -m "chore: prepare release vX.Y.Z"
   git push
   ```

## Release

4. **Create and push tag**
   ```bash
   git tag vX.Y.Z
   git push origin vX.Y.Z
   ```

5. **Wait for GitHub Actions**
   - The `.github/workflows/release.yml` will automatically build and upload binaries
   - Check: https://github.com/nixval/declarch/actions

6. **Verify release assets**
   ```bash
   # Check that binaries were uploaded
   gh release view vX.Y.Z --json assets --jq '.assets[].name'

   # Download and test the binary
   wget https://github.com/nixval/declarch/releases/download/vX.Y.Z/declarch-x86_64-unknown-linux-gnu.tar.gz
   tar xzf declarch-x86_64-unknown-linux-gnu.tar.gz
   ./declarch --version  # Should show X.Y.Z
   ```

7. **Create GitHub release (if auto-create fails)**
   ```bash
   gh release create vX.Y.Z \
     --title "Declarch X.Y.Z" \
     --notes-file CHANGELOG.md
   ```

## Post-Release

8. **Test install script**
   ```bash
   # Remove existing installations first
   sudo rm -f /usr/local/bin/declarch /usr/bin/declarch

   # Test install from main
   curl -fsSL https://raw.githubusercontent.com/nixval/declarch/main/install.sh | sh

   # Verify version
   declarch --version  # Should show X.Y.Z
   ```

9. **AUR update (optional)**
   ```bash
   cd .aur
   ./publish.sh
   ```

## Common Issues & Solutions

### Issue: Binary shows wrong version
**Cause**: Built before version bump
**Solution**:
```bash
cargo clean --release
cargo build --release
./target/release/declarch --version  # Verify
```

### Issue: install.sh downloads 404
**Cause**: Release created without binary artifacts
**Solution**: Build and upload manually:
```bash
cargo build --release
cd target/release
tar czf ../../declarch-x86_64-unknown-linux-gnu.tar.gz declarch
gh release upload vX.Y.Z ../../declarch-x86_64-unknown-linux-gnu.tar.gz
```

### Issue: User has multiple declarch installations
**Cause**: Old binaries in PATH
**Solution**: The install.sh now detects and warns about this

### Issue: CI fails on cargo fmt
**Cause**: Formatting mismatch
**Solution**:
```bash
cargo fmt
git add -A
git commit -m "style: apply cargo fmt"
git push
```

### Issue: CI fails on clippy
**Cause**: Code warnings
**Solution**: Fix all clippy warnings locally first

### Issue: Doctests fail after adding struct fields
**Cause**: Examples in documentation need updating
**Solution**: Update all doctests to include new struct fields

## Quick Release Script

Save this as `scripts/release.sh`:

```bash
#!/bin/bash
set -e

VERSION=$1
if [ -z "$VERSION" ]; then
    echo "Usage: ./scripts/release.sh X.Y.Z"
    exit 1
fi

echo "🔄 Preparing release $VERSION..."

# Update Cargo.toml
sed -i "s/^version = .*/version = \"$VERSION\"/" Cargo.toml

# Update install.sh
sed -i "s/^VERSION=\".*\"/VERSION=\"$VERSION\"/" install.sh

# Commit and push
git add -A
git commit -m "chore: prepare release $VERSION"
git push

# Create tag
echo "🏷️  Creating tag v$VERSION..."
git tag "v$VERSION"
git push origin "v$VERSION"

echo "✅ Release $VERSION prepared!"
echo "🔗 Check CI: https://github.com/nixval/declarch/actions"
echo "📦 Verify release at: https://github.com/nixval/declarch/releases"
```

Usage: `./scripts/release.sh 0.5.3`
