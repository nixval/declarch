#!/usr/bin/env bash
#
# Publish declarch to AUR
# Usage: ./publish-declarch.sh [version]
# Example: ./publish-declarch.sh 0.4.0
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

# Create temporary directory
TMP_DIR=$(mktemp -d)
DECLARCH_DIR="$TMP_DIR/declarch"

cleanup() {
    echo "Cleaning up..."
    rm -rf "$TMP_DIR"
}
trap cleanup EXIT

# Prepare declarch AUR package
echo "Preparing declarch AUR package..."
mkdir -p "$DECLARCH_DIR"
cp PKGBUILD-declarch "$DECLARCH_DIR/PKGBUILD"
sed -i "s/pkgver=.*/pkgver=${VERSION}/" "$DECLARCH_DIR/PKGBUILD"

# Generate .SRCINFO
echo "Generating .SRCINFO..."
cd "$DECLARCH_DIR"
makepkg --printsrcinfo > .SRCINFO

# Show what will be published
echo ""
echo "=== declarch AUR Package ==="
echo "Files:"
ls -la "$DECLARCH_DIR/"

# Confirm before publishing
read -p "Publish declarch to AUR? (y/N) " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Aborted."
    exit 0
fi

# Publish to AUR
echo ""
echo "Publishing declarch to AUR..."
git init
git config user.name "nixval"
git config user.email "nicovaliantoku@gmail.com"
git checkout -b master
git add PKGBUILD .SRCINFO
git commit -m "Update to ${VERSION}"
git remote add origin "$AUR_REMOTE:declarch.git"
git push -u origin master --force

echo ""
echo "✓ Successfully published declarch v${VERSION} to AUR!"
