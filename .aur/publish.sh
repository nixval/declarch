#!/usr/bin/env bash
#
# Publish declarch to AUR
# Usage: ./publish.sh [version]
# Example: ./publish.sh 0.4.0
#

set -e

VERSION=${1:-$(cargo pkgid | cut -d# -f2 | cut -d: -f2)}
AUR_REMOTE="aur@aur.archlinux.org"

if [[ -z "$VERSION" ]]; then
    echo "Error: Could not determine version"
    echo "Usage: $0 [version]"
    exit 1
fi

echo "Publishing declarch v${VERSION} to AUR..."

# Create temporary directories
TMP_DIR=$(mktemp -d)
DECLARCH_DIR="$TMP_DIR/declarch"
DECLARCH_BIN_DIR="$TMP_DIR/declarch-bin"

cleanup() {
    echo "Cleaning up..."
    rm -rf "$TMP_DIR"
}
trap cleanup EXIT

# Prepare declarch AUR package
echo "Preparing declarch AUR package..."
mkdir -p "$DECLARCH_DIR"
cp .aur/PKGBUILD-declarch "$DECLARCH_DIR/PKGBUILD"
sed -i "s/pkgver=.*/pkgver=${VERSION}/" "$DECLARCH_DIR/PKGBUILD"

# Prepare declarch-bin AUR package
echo "Preparing declarch-bin AUR package..."
mkdir -p "$DECLARCH_BIN_DIR"
cp .aur/PKGBUILD-declarch-bin "$DECLARCH_BIN_DIR/PKGBUILD"
sed -i "s/pkgver=.*/pkgver=${VERSION}/" "$DECLARCH_BIN_DIR/PKGBUILD"

# Generate .SRCINFO for declarch
echo "Generating .SRCINFO for declarch..."
cd "$DECLARCH_DIR"
makepkg --printsrcinfo > .SRCINFO
cd - > /dev/null

# Generate .SRCINFO for declarch-bin
echo "Generating .SRCINFO for declarch-bin..."
cd "$DECLARCH_BIN_DIR"
makepkg --printsrcinfo > .SRCINFO
cd - > /dev/null

# Show what will be published
echo ""
echo "=== declarch AUR Package ==="
echo "$DECLARCH_DIR/"
echo "Files:"
ls -la "$DECLARCH_DIR/"

echo ""
echo "=== declarch-bin AUR Package ==="
echo "$DECLARCH_BIN_DIR/"
echo "Files:"
ls -la "$DECLARCH_BIN_DIR/"

# Confirm before publishing
read -p "Publish to AUR? (y/N) " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Aborted."
    exit 0
fi

# Publish declarch
echo ""
echo "Publishing declarch..."
cd "$DECLARCH_DIR"
git init
git add PKGBUILD .SRCINFO
git commit -m "Update to ${VERSION}"
git remote add origin "$AUR_REMOTE:declarch.git"
git push -u origin main --force

# Publish declarch-bin
echo ""
echo "Publishing declarch-bin..."
cd "$DECLARCH_BIN_DIR"
git init
git add PKGBUILD .SRCINFO
git commit -m "Update to ${VERSION}"
git remote add origin "$AUR_REMOTE:declarch-bin.git"
git push -u origin main --force

echo ""
echo "✓ Successfully published to AUR!"
echo "  - declarch v${VERSION}"
echo "  - declarch-bin v${VERSION}"
