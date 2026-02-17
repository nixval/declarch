#!/usr/bin/env bash
#
# Helper script to build individual AUR packages
# This makes it easy to test each PKGBUILD separately
#
# Usage: ./build-aur.sh [package]
# Examples:
#   ./build-aur.sh           # Lists available packages
#   ./build-aur.sh declarch  # Builds declarch package
#   ./build-aur.sh bin       # Builds declarch-bin package
#   ./build-aur.sh git       # Builds declarch-git package
#

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Available packages
declare -A PACKAGES=(
    ["declarch"]=".aur/work/declarch"
    ["declarch-bin"]=".aur/work/declarch-bin"
    ["declarch-git"]=".aur/work/declarch-git"
)

# Get version from Cargo.toml
VERSION=$(grep "^version" ../Cargo.toml 2>/dev/null | head -1 | sed 's/version = "\(.*\)"/\1/')
if [[ -z "$VERSION" ]]; then
    VERSION=$(grep "^version" Cargo.toml 2>/dev/null | head -1 | sed 's/version = "\(.*\)"/\1/')
fi

# Function to build a package
build_package() {
    local pkg_name=$1
    local pkg_dir=$2

    echo -e "${BLUE}========================================${NC}"
    echo -e "${BLUE}  Building ${pkg_name}${NC}"
    echo -e "${BLUE}========================================${NC}"
    echo ""

    # Check if PKGBUILD exists
    if [[ ! -f "$pkg_dir/PKGBUILD" ]]; then
        echo -e "${YELLOW}PKGBUILD not found. Preparing from template...${NC}"

        case $pkg_name in
            "declarch")
                sed "s/pkgver=.*/pkgver=${VERSION}/" .aur/templates/PKGBUILD > "$pkg_dir/PKGBUILD"
                cd "$pkg_dir"
                makepkg -g >> PKGBUILD
                cd - > /dev/null
                ;;
            "declarch-bin")
                sed "s/pkgver=.*/pkgver=${VERSION}/" .aur/templates/PKGBUILD-bin > "$pkg_dir/PKGBUILD"
                echo -e "${YELLOW}Note: declarch-bin requires GitHub release to be available${NC}"
                ;;
            "declarch-git")
                cp .aur/templates/PKGBUILD-git "$pkg_dir/PKGBUILD"
                ;;
        esac
        echo -e "${GREEN}✓ PKGBUILD prepared${NC}"
        echo ""
    fi

    # Clean previous build
    echo -e "${YELLOW}Cleaning previous build...${NC}"
    cd "$pkg_dir"
    rm -rf pkg src *.tar.gz 2>/dev/null || true
    cd - > /dev/null
    echo -e "${GREEN}✓ Cleaned${NC}"
    echo ""

    # Build package
    echo -e "${YELLOW}Building package...${NC}"
    cd "$pkg_dir"

    case $pkg_name in
        "declarch"|"declarch-git")
            makepkg -sf
            ;;
        "declarch-bin")
            makepkg -sf
            ;;
    esac

    echo -e "${GREEN}✓ Package built successfully!${NC}"
    echo ""

    # Show package info
    PKG_FILE=$(ls -t *.tar.gz 2>/dev/null | head -1)
    if [[ -n "$PKG_FILE" ]]; then
        PKG_SIZE=$(du -h "$PKG_FILE" | cut -f1)
        echo -e "${BLUE}Package file:${NC} $PKG_FILE"
        echo -e "${BLUE}Package size:${NC} $PKG_SIZE"
        echo ""

        # Ask if user wants to install
        echo -e "${YELLOW}Install this package? (y/N)${NC}"
        read -r -n 1 response
        echo ""
        if [[ $response =~ ^[Yy]$ ]]; then
            echo -e "${YELLOW}Installing package...${NC}"
            sudo pacman -U "$PKG_FILE"
            echo -e "${GREEN}✓ Package installed${NC}"
        fi
    fi

    cd - > /dev/null
}

# Main script
if [[ -z "$1" ]]; then
    echo -e "${BLUE}Available AUR packages:${NC}"
    echo ""
    for pkg in "${!PACKAGES[@]}"; do
        echo "  - $pkg"
    done
    echo ""
    echo "Usage: $0 [package]"
    echo "Example: $0 declarch"
    exit 0
fi

# Match package name (allow short names)
PKG_ARG="$1"
PKG_NAME=""
for pkg in "${!PACKAGES[@]}"; do
    if [[ "$pkg" == *"$PKG_ARG"* ]] || [[ "$PKG_ARG" == "bin" && "$pkg" == "declarch-bin" ]] || [[ "$PKG_ARG" == "git" && "$pkg" == "declarch-git" ]]; then
        PKG_NAME="$pkg"
        PKG_DIR="${PACKAGES[$pkg]}"
        break
    fi
done

if [[ -z "$PKG_NAME" ]]; then
    echo -e "${RED}Error: Unknown package '$1'${NC}"
    echo ""
    echo "Available packages:"
    for pkg in "${!PACKAGES[@]}"; do
        echo "  - $pkg"
    done
    exit 1
fi

# Create work directory if it doesn't exist
mkdir -p "$(dirname "$PKG_DIR")"

# Build the package
build_package "$PKG_NAME" "$PKG_DIR"
