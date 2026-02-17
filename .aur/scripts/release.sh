#!/usr/bin/env bash
#
# Complete Release Automation
# This script handles the entire release process automatically
#
# Usage: ./release.sh [version]
# Example: ./release.sh 0.4.0
#

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Get version from argument or Cargo.toml
VERSION=${1}
if [[ -z "$VERSION" ]]; then
    VERSION=$(grep "^version" Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
fi

if [[ -z "$VERSION" ]]; then
    echo -e "${RED}Error: Could not determine version${NC}"
    exit 1
fi

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}  Declarch Release Automation v${VERSION}${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

# Step 1: Pre-release checks
echo -e "${YELLOW}[1/7] Pre-release checks...${NC}"
if [[ -n $(git status --porcelain) ]]; then
    echo -e "${RED}Error: Working directory is not clean${NC}"
    echo "Please commit or stash changes first"
    exit 1
fi

# Run tests
echo "Running tests..."
if ! cargo test --quiet 2>&1 >/dev/null; then
    echo -e "${RED}Error: Tests failed${NC}"
    exit 1
fi
echo -e "${GREEN}✓ Tests passed${NC}"
echo ""

# Step 2: Build release binary
echo -e "${YELLOW}[2/7] Building release binary with LTO optimization...${NC}"
CARGO_PROFILE_RELEASE_LTO=thin cargo build --release --quiet
echo -e "${GREEN}✓ Binary built (with LTO optimization)${NC}"
echo ""

# Step 3: Create work directory and prepare PKGBUILDs
echo -e "${YELLOW}[3/7] Preparing PKGBUILDs...${NC}"
WORK_DIR=".aur/work"
rm -rf "$WORK_DIR"
mkdir -p "$WORK_DIR"/{declarch,declarch-bin,declarch-git}

# Prepare declarch PKGBUILD
echo "Preparing declarch PKGBUILD..."
sed "s/pkgver=.*/pkgver=${VERSION}/" .aur/templates/PKGBUILD > "$WORK_DIR/declarch/PKGBUILD"
cd "$WORK_DIR/declarch"
makepkg -g >> PKGBUILD
cd - > /dev/null

# Prepare declarch-bin PKGBUILD
echo "Preparing declarch-bin PKGBUILD..."
sed "s/pkgver=.*/pkgver=${VERSION}/" .aur/templates/PKGBUILD-bin > "$WORK_DIR/declarch-bin/PKGBUILD"
# Note: SHA256 will be generated later after GitHub release

# Prepare declarch-git PKGBUILD
echo "Preparing declarch-git PKGBUILD..."
cp .aur/templates/PKGBUILD-git "$WORK_DIR/declarch-git/PKGBUILD"

echo -e "${GREEN}✓ PKGBUILDs prepared${NC}"
echo ""

# Step 4: Test PKGBUILDs
echo -e "${YELLOW}[4/7] Testing PKGBUILDs...${NC}"

# Test declarch
echo "Testing declarch PKGBUILD..."
cd "$WORK_DIR/declarch"
if ! makepkg -s; then
    echo -e "${RED}Error: declarch PKGBUILD test failed${NC}"
    exit 1
fi
echo -e "${GREEN}✓ declarch PKGBUILD OK${NC}"
cd - > /dev/null

# Test declarch-git
echo "Testing declarch-git PKGBUILD..."
cd "$WORK_DIR/declarch-git"
if ! makepkg --nobuild; then
    echo -e "${RED}Error: declarch-git PKGBUILD test failed${NC}"
    exit 1
fi
echo -e "${GREEN}✓ declarch-git PKGBUILD OK${NC}"
cd - > /dev/null

echo -e "${GREEN}✓ All PKGBUILDs tested${NC}"
echo ""

# Step 5: Commit and push to main
echo -e "${YELLOW}[5/7] Committing and pushing to main...${NC}"

# Copy PKGBUILD to root (the only one that should be in git)
cp "$WORK_DIR/declarch/PKGBUILD" PKGBUILD

# Update templates locally (not tracked in git)
cp "$WORK_DIR/declarch/PKGBUILD" .aur/templates/PKGBUILD
cp "$WORK_DIR/declarch-git/PKGBUILD" .aur/templates/PKGBUILD-git

# Add and commit only root PKGBUILD
git add PKGBUILD
git commit -m "chore: update PKGBUILD for v${VERSION}"
git push origin main

echo -e "${GREEN}✓ Pushed to main${NC}"
echo ""

# Step 6: Create and push tag
echo -e "${YELLOW}[6/7] Creating and pushing tag...${NC}"

# Check if tag exists
if git rev-parse "v${VERSION}" >/dev/null 2>&1; then
    echo -e "${YELLOW}Warning: Tag v${VERSION} already exists${NC}"
    read -p "Delete and recreate? (y/N) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        git tag -d "v${VERSION}"
        git push origin ":refs/tags/v${VERSION}"
    else
        echo "Aborted."
        exit 0
    fi
fi

# Create tag
git tag -a "v${VERSION}" -m "Release v${VERSION}"
git push origin "v${VERSION}"

echo -e "${GREEN}✓ Tag v${VERSION} pushed${NC}"
echo ""

# Step 7: Wait for GitHub release
echo -e "${YELLOW}[7/7] Waiting for GitHub release...${NC}"
echo ""
echo -e "${BLUE}Next steps:${NC}"
echo "1. Wait for GitHub Actions to complete (±10 minutes)"
echo "   Monitor at: https://github.com/nixval/declarch/actions"
echo ""
echo "2. After GitHub Actions finishes, run:"
echo "   ./.aur/scripts/publish.sh ${VERSION}"
echo ""
echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}  Release preparation complete!${NC}"
echo -e "${GREEN}========================================${NC}"
