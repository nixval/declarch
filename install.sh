#!/bin/bash
# declarch Installation Script
# Automatically installs declarch and Soar package manager

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
    # Installation from crates.io or git
    INSTALL_METHOD="${1:-cargo}"

    if [ "$INSTALL_METHOD" = "cargo" ]; then
        echo -e "${BLUE}📦 Installing declarch via cargo...${NC}"
        cargo install declarch
    elif [ "$INSTALL_METHOD" = "git" ]; then
        echo -e "${BLUE}📦 Installing declarch from git...${NC}"
        cargo install declarch --git https://github.com/user/declarch
    else
        echo -e "${RED}Error: Unknown installation method '$INSTALL_METHOD'${NC}"
        echo "Usage: $0 [cargo|git|--local]"
        exit 1
    fi
fi

echo -e "${GREEN}✅ declarch installed successfully${NC}"

# Check if Soar is already installed
if command -v soar &> /dev/null; then
    echo -e "${GREEN}✅ Soar already installed${NC}"
    SOAR_VERSION=$(soar --version 2>/dev/null || echo "unknown")
    echo -e "   Version: ${SOAR_VERSION}"
else
    echo -e "${BLUE}📦 Installing Soar (cross-distro package manager)...${NC}"

    # Try curl first, then wget
    if command -v curl &> /dev/null; then
        curl -fsSL "https://raw.githubusercontent.com/pkgforge/soar/main/install.sh" | sh
    elif command -v wget &> /dev/null; then
        wget -qO- "https://raw.githubusercontent.com/pkgforge/soar/main/install.sh" | sh
    else
        echo -e "${RED}Error: Neither curl nor wget found${NC}"
        echo -e "${YELLOW}Please install Soar manually:${NC}"
        echo "   https://github.com/pkgforge/soar#installation"
        exit 1
    fi

    # Verify Soar installation
    if command -v soar &> /dev/null; then
        echo -e "${GREEN}✅ Soar installed successfully${NC}"
        SOAR_VERSION=$(soar --version 2>/dev/null || echo "unknown")
        echo -e "   Version: ${SOAR_VERSION}"
    else
        echo -e "${RED}❌ Soar installation failed${NC}"
        echo -e "${YELLOW}Please install Soar manually:${NC}"
        echo "   https://github.com/pkgforge/soar#installation"
        exit 1
    fi
fi

echo ""
echo -e "${BLUE}🔍 Checking for additional package managers...${NC}"

# Check for AUR helpers (Arch Linux)
if command -v paru &> /dev/null; then
    echo -e "  ${GREEN}✅${NC} paru found ${GREEN}(AUR support enabled)${NC}"
elif command -v yay &> /dev/null; then
    echo -e "  ${GREEN}✅${NC} yay found ${GREEN}(AUR support enabled)${NC}"
elif [ -f /etc/arch-release ]; then
    echo -e "  ${YELLOW}⚠️  No AUR helper found${NC}"
    echo -e "     ${YELLOW}Install paru or yay for AUR support:${NC}"
    echo "     paru: https://github.com/Morganamilo/paru#install"
    echo "     yay: https://github.com/Jguer/yay#installation"
fi

# Check for Flatpak
if command -v flatpak &> /dev/null; then
    echo -e "  ${GREEN}✅${NC} flatpak found ${GREEN}(Flatpak support enabled)${NC}"
else
    echo -e "  ${YELLOW}⚠️  flatpak not found${NC}"
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
echo -e "      ${BLUE}~/.config/declarch/packages.kdl${NC}"
echo ""
echo "   3. Sync packages:"
echo -e "      ${BLUE}declarch sync${NC}"
echo ""
echo -e "${BLUE}Documentation:${NC}"
echo "   https://github.com/user/declarch#readme"
