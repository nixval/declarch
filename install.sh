#!/bin/bash
# declarch Installation Script
# Installs declarch package manager

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🚀 Installing declarch...${NC}"

# Detect installation method
if [ "$1" = "--local" ]; then
    # Local installation from current directory
    echo -e "${BLUE}📦 Installing declarch from source...${NC}"

    if [ ! -f "Cargo.toml" ]; then
        echo -e "${RED}Error: Cargo.toml not found. Are you in the declarch directory?${NC}"
        exit 1
    fi

    cargo build --release
    sudo cp target/release/declarch /usr/local/bin/
    sudo chmod +x /usr/local/bin/declarch

else
    # Installation from git (default) or local
    INSTALL_METHOD="${1:-git}"

    if [ "$INSTALL_METHOD" = "git" ]; then
        echo -e "${BLUE}📦 Installing declarch from git...${NC}"
        cargo install declarch --git https://github.com/nixval/declarch --tag v0.4.3
    else
        echo -e "${RED}Error: Unknown installation method '$INSTALL_METHOD'${NC}"
        echo "Usage: $0 [git|--local]"
        exit 1
    fi
fi

echo -e "${GREEN}✅ declarch installed successfully${NC}"

echo ""
echo -e "${BLUE}🔍 Optional Dependencies:${NC}"
echo ""

# Check for Soar
if command -v soar &> /dev/null; then
    echo -e "  ${GREEN}✅${NC} Soar found ${GREEN}(cross-distro packages enabled)${NC}"
else
    echo -e "  ${YELLOW}○ Soar not found${NC}"
    echo -e "     ${YELLOW}Install for cross-distro static packages:${NC}"
    echo "     https://github.com/pkgforge/soar#installation"
fi

# Check for AUR helpers (Arch Linux)
if command -v paru &> /dev/null; then
    echo -e "  ${GREEN}✅${NC} paru found ${GREEN}(AUR support enabled)${NC}"
elif command -v yay &> /dev/null; then
    echo -e "  ${GREEN}✅${NC} yay found ${GREEN}(AUR support enabled)${NC}"
elif [ -f /etc/arch-release ]; then
    echo -e "  ${YELLOW}○ No AUR helper found${NC}"
    echo -e "     ${YELLOW}Install for AUR support:${NC}"
    echo "     paru: https://github.com/Morganamilo/paru#install"
    echo "     yay: https://github.com/Jguer/yay#installation"
fi

# Check for Flatpak
if command -v flatpak &> /dev/null; then
    echo -e "  ${GREEN}✅${NC} flatpak found ${GREEN}(Flatpak support enabled)${NC}"
else
    echo -e "  ${YELLOW}○ flatpak not found${NC}"
    echo -e "     ${YELLOW}Install for Flatpak support:${NC}"

    # Detect distro and provide appropriate command
    if [ -f /etc/arch-release ]; then
        echo "        sudo pacman -S flatpak"
    elif [ -f /etc/debian_version ]; then
        echo "        sudo apt install flatpak"
    elif [ -f /etc/fedora-release ]; then
        echo "        sudo dnf install flatpak"
    fi
fi

echo ""
echo -e "${GREEN}✨ Installation complete!${NC}"
echo ""
echo -e "${BLUE}Next steps:${NC}"
echo "   1. Initialize declarch:"
echo -e "      ${BLUE}declarch init${NC}"
echo ""
echo "   2. Create your configuration:"
echo -e "      ${BLUE}~/.config/declarch/declarch.kdl${NC}"
echo ""
echo "   3. Sync packages:"
echo -e "      ${BLUE}declarch sync${NC}"
echo ""
echo -e "${BLUE}Documentation:${NC}"
echo "   https://github.com/nixval/declarch#readme"
echo ""
