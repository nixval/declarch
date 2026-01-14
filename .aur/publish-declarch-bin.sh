#!/usr/bin/env bash
#
# Publish declarch-bin to AUR
# Usage: ./publish-declarch-bin.sh [version]
# Example: ./publish-declarch-bin.sh 0.4.0
#

set -e

VERSION=${1:-$(cargo pkgid | cut -d# -f2 | cut -d: -f2)}
AUR_REMOTE="aur@aur.archlinux.org"

if [[ -z "$VERSION" ]]; then
    echo "Error: Could not determine version"
    echo "Usage: $0 [version]"
    exit 1
fi

echo "Publishing declarch-bin v${VERSION} to AUR..."

# Create temporary directory
TMP_DIR=$(mktemp -d)
DECLARCH_BIN_DIR="$TMP_DIR/declarch-bin"

cleanup() {
    echo "Cleaning up..."
    rm -rf "$TMP_DIR"
}
trap cleanup EXIT

# Prepare declarch-bin AUR package
echo "Preparing declarch-bin AUR package..."
mkdir -p "$DECLARCH_BIN_DIR"
cp PKGBUILD-bin "$DECLARCH_BIN_DIR/PKGBUILD"
sed -i "s/pkgver=.*/pkgver=${VERSION}/" "$DECLARCH_BIN_DIR/PKGBUILD"

# Generate .SRCINFO
echo "Generating .SRCINFO..."
cd "$DECLARCH_BIN_DIR"
makepkg --printsrcinfo > .SRCINFO

# Show what will be published
echo ""
echo "=== declarch-bin AUR Package ==="
echo "Files:"
ls -la "$DECLARCH_BIN_DIR/"

# Confirm before publishing
read -p "Publish declarch-bin to AUR? (y/N) " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Aborted."
    exit 0
fi

# Publish to AUR
echo ""
echo "Publishing declarch-bin to AUR..."
git init
git config user.name "nixval"
git config user.email "nicovaliantoku@gmail.com"
git checkout -b master
git add PKGBUILD .SRCINFO
git commit -m "Update to ${VERSION}"
git remote add origin "$AUR_REMOTE:declarch-bin.git"
git push -u origin master --force

echo ""
echo "✓ Successfully published declarch-bin v${VERSION} to AUR!"
