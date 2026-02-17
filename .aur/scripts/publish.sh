#!/usr/bin/env bash
#
# AUR Publish Automation
# Publishes all packages to AUR after GitHub release is complete
#
# Usage: ./publish.sh [version]
# Example: ./publish.sh 0.4.0
#

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Get version
VERSION=${1}
if [[ -z "$VERSION" ]]; then
    VERSION=$(grep "^version" Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
fi

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}  AUR Publish Automation v${VERSION}${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

# Verify GitHub release exists
echo -e "${YELLOW}[1/5] Verifying GitHub release...${NC}"
if ! gh release view "v${VERSION}" >/dev/null 2>&1; then
    echo -e "${RED}Error: GitHub release v${VERSION} not found${NC}"
    echo ""
    echo "Please wait for GitHub Actions to complete first:"
    echo "  https://github.com/nixval/declarch/actions"
    exit 1
fi
echo -e "${GREEN}✓ GitHub release exists${NC}"
echo ""

# Download binary release and update declarch-bin PKGBUILD
echo -e "${YELLOW}[2/5] Updating declarch-bin PKGBUILD...${NC}"

# Setup work directory
WORK_DIR=".aur/work"
mkdir -p "$WORK_DIR"/{declarch,declarch-bin,declarch-git}

# Copy templates if not already present
if [[ ! -f "$WORK_DIR/declarch/PKGBUILD" ]]; then
    sed "s/pkgver=.*/pkgver=${VERSION}/" .aur/templates/PKGBUILD > "$WORK_DIR/declarch/PKGBUILD"
    cd "$WORK_DIR/declarch"
    makepkg -g >> PKGBUILD
    cd - > /dev/null
fi

if [[ ! -f "$WORK_DIR/declarch-bin/PKGBUILD" ]]; then
    sed "s/pkgver=.*/pkgver=${VERSION}/" .aur/templates/PKGBUILD-bin > "$WORK_DIR/declarch-bin/PKGBUILD"
fi

if [[ ! -f "$WORK_DIR/declarch-git/PKGBUILD" ]]; then
    cp .aur/templates/PKGBUILD-git "$WORK_DIR/declarch-git/PKGBUILD"
fi

cd "$WORK_DIR/declarch-bin"
wget -q "https://github.com/nixval/declarch/releases/download/v${VERSION}/declarch-x86_64-unknown-linux-gnu.tar.gz"

# Generate sha256
SHA256=$(sha256sum declarch-x86_64-unknown-linux-gnu.tar.gz | awk '{print $1}')
echo "Binary SHA256: $SHA256"

# Update PKGBUILD with correct sha256
sed "s/sha256sums=.*/sha256sums=('$SHA256')/" PKGBUILD > PKGBUILD.tmp
mv PKGBUILD.tmp PKGBUILD

cd - > /dev/null
echo -e "${GREEN}✓ declarch-bin PKGBUILD updated${NC}"
echo ""

# Test build declarch-bin
echo -e "${YELLOW}[3/5] Testing declarch-bin build...${NC}"
cd "$WORK_DIR/declarch-bin"
rm -rf pkg src
if ! makepkg; then
    echo -e "${RED}Error: declarch-bin build failed${NC}"
    exit 1
fi
echo -e "${GREEN}✓ declarch-bin builds successfully${NC}"
cd - > /dev/null
echo ""

# Publish to AUR
echo -e "${YELLOW}[4/5] Publishing to AUR...${NC}"

# Function to publish single package
publish_package() {
    local pkg_name=$1
    local pkg_dir=$2
    
    echo "Publishing ${pkg_name}..."
    
    cd "$pkg_dir"
    
    # Generate .SRCINFO
    makepkg --printsrcinfo > .SRCINFO
    
    # Check if this is a fresh clone or update
    AUR_DIR="/tmp/aur-${pkg_name}-${VERSION}"
    if [[ -d "$AUR_DIR" ]]; then
        rm -rf "$AUR_DIR"
    fi
    
    mkdir -p "$AUR_DIR"
    cp PKGBUILD .SRCINFO "$AUR_DIR/"
    
    cd "$AUR_DIR"
    
    # Initialize git repo
    git init
    git config user.name "nixval"
    git config user.email "nicovaliantoku@gmail.com"
    
    # Clone existing AUR repo if it exists to preserve history
    if git ls-remote "aur@aur.archlinux.org:${pkg_name}.git" HEAD &>/dev/null; then
        echo "  Cloning existing AUR repo..."
        git clone "aur@aur.archlinux.org:${pkg_name}.git" .aur_temp || true
        if [[ -d .aur_temp/.git ]]; then
            mv .aur_temp/.git .
            rm -rf .aur_temp
            git reset --hard HEAD
        fi
    fi
    
    # Add files
    git add PKGBUILD .SRCINFO
    
    # Check if there are changes
    if git diff --cached --quiet; then
        echo "  No changes to commit for ${pkg_name}"
        cd - > /dev/null
        return
    fi
    
    # Commit
    git commit -m "Update to ${VERSION}"
    
    # Push
    echo "  Pushing to AUR..."
    if git push -f "aur@aur.archlinux.org:${pkg_name}.git" HEAD:master 2>&1; then
        echo -e "${GREEN}  ✓ ${pkg_name} published${NC}"
    else
        echo -e "${RED}  ✗ ${pkg_name} publish failed${NC}"
        cd - > /dev/null
        return 1
    fi
    
    cd - > /dev/null
}

# Publish all three packages
publish_package "declarch" "$WORK_DIR/declarch"
publish_package "declarch-bin" "$WORK_DIR/declarch-bin"
publish_package "declarch-git" "$WORK_DIR/declarch-git"

echo ""
echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}  All packages published to AUR!${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""
echo "Packages:"
echo "  - declarch: https://aur.archlinux.org/packages/declarch"
echo "  - declarch-bin: https://aur.archlinux.org/packages/declarch-bin"
echo "  - declarch-git: https://aur.archlinux.org/packages/declarch-git"
