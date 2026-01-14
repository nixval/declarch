#!/usr/bin/env bash
#
# Publish declarch-git to AUR
# Usage: ./publish-declarch-git.sh
#

set -e

AUR_REMOTE="aur@aur.archlinux.org"

echo "Publishing declarch-git to AUR (latest git version)..."

# Create temporary directory
TMP_DIR=$(mktemp -d)
DECLARCH_GIT_DIR="$TMP_DIR/declarch-git"

cleanup() {
    echo "Cleaning up..."
    rm -rf "$TMP_DIR"
}
trap cleanup EXIT

# Prepare declarch-git AUR package
echo "Preparing declarch-git AUR package..."
mkdir -p "$DECLARCH_GIT_DIR"
cp PKGBUILD-declarch-git "$DECLARCH_GIT_DIR/PKGBUILD"

# Generate .SRCINFO
echo "Generating .SRCINFO..."
cd "$DECLARCH_GIT_DIR"
makepkg --printsrcinfo > .SRCINFO

# Show what will be published
echo ""
echo "=== declarch-git AUR Package ==="
echo "Files:"
ls -la "$DECLARCH_GIT_DIR/"

# Confirm before publishing
read -p "Publish declarch-git to AUR? (y/N) " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Aborted."
    exit 0
fi

# Publish to AUR
echo ""
echo "Publishing declarch-git to AUR..."
git init
git config user.name "nixval"
git config user.email "nicovaliantoku@gmail.com"
git checkout -b master
git add PKGBUILD .SRCINFO
git commit -m "Initial commit"
git remote add origin "$AUR_REMOTE:declarch-git.git"
git push -u origin master --force

echo ""
echo "✓ Successfully published declarch-git to AUR!"
