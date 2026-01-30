#!/bin/bash
# declarch installer script
# Usage: curl --proto '=https' --tlsv1.2 -sSf https://nixval.github.io/declarch/install.sh | sh

set -e

VERSION="0.5.1"
REPO="nixval/declarch"
BINARY_NAME="declarch"
INSTALL_DIR="/usr/local/bin"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

info() {
    echo -e "${GREEN}➜${NC} $1"
}

error() {
    echo -e "${RED}✗${NC} $1" >&2
}

warn() {
    echo -e "${YELLOW}⚠${NC} $1"
}

# Detect platform
ARCH=$(uname -m)
OS=$(uname -s)

case "$ARCH" in
    x86_64|amd64)
        ARCH="x86_64"
        ;;
    aarch64|arm64)
        ARCH="aarch64"
        ;;
    *)
        error "Unsupported architecture: $ARCH"
        exit 1
        ;;
esac

# Determine binary name and download URL
BINARY_FILE="${BINARY_NAME}-${ARCH}-unknown-linux-gnu"

if [ "$OS" != "Linux" ]; then
    error "This script only supports Linux. For macOS, use Homebrew"
    exit 1
fi

DOWNLOAD_URL="https://github.com/${REPO}/releases/download/v${VERSION}/${BINARY_FILE}.tar.gz"

echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}🚀 Installing declarch v${VERSION}${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
info "Platform: Linux ${ARCH}"
info "Download: $DOWNLOAD_URL"
echo ""

# Create temp directory
TMP_DIR=$(mktemp -d)
trap "rm -rf $TMP_DIR" EXIT

cd "$TMP_DIR"

# Download
if command -v wget >/dev/null 2>&1; then
    wget -q --show-progress "$DOWNLOAD_URL" -O "${BINARY_FILE}.tar.gz"
elif command -v curl >/dev/null 2>&1; then
    curl -# -L "$DOWNLOAD_URL" -o "${BINARY_FILE}.tar.gz"
else
    error "Neither wget nor curl is installed"
    exit 1
fi

# Extract
tar xzf "${BINARY_FILE}.tar.gz"

# Check if binary exists
if [ ! -f "$BINARY_NAME" ]; then
    error "Failed to extract binary"
    exit 1
fi

# Verify version
if [ -f "$BINARY_NAME" ]; then
    echo ""
    info "Downloaded binary:"
    ./"$BINARY_NAME" --version 2>/dev/null || true
    echo ""
fi

# Install
if [ -w "$INSTALL_DIR" ]; then
    info "Installing to $INSTALL_DIR..."
    mv "$BINARY_NAME" "$INSTALL_DIR/"
    chmod +x "$INSTALL_DIR/$BINARY_NAME"

    # Also install dcl symlink if exists
    if [ -f "dcl" ]; then
        mv "dcl" "$INSTALL_DIR/"
        chmod +x "$INSTALL_DIR/dcl"
    fi
else
    warn "No write permission for $INSTALL_DIR"
    info "Installing to ~/.local/bin instead..."
    mkdir -p ~/.local/bin
    mv "$BINARY_NAME" ~/.local/bin/
    chmod +x ~/.local/bin/"$BINARY_NAME"

    if [ -f "dcl" ]; then
        mv "dcl" ~/.local/bin/
        chmod +x ~/.local/bin/dcl
    fi

    # Add to PATH if not already there
    if [[ ":$PATH:" != *":$HOME/.local/bin:"* ]]; then
        warn "~/.local/bin is not in PATH"
        echo ""
        info "Add this to your ~/.bashrc or ~/.zshrc:"
        echo -e "${GREEN}  export PATH=\"\$HOME/.local/bin:\$PATH\"${NC}"
    fi
fi

# Cleanup
cd - >/dev/null
rm -rf "$TMP_DIR"

echo ""
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}✅ Installation complete!${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
echo -e "${BLUE}Next steps:${NC}"
echo -e "  1. Initialize config:"
echo -e "     ${GREEN}declarch init${NC}"
echo ""
echo -e "  2. Install packages:"
echo -e "     ${GREEN}declarch install bat fzf ripgrep${NC}"
echo ""
echo -e "  3. Sync system:"
echo -e "     ${GREEN}declarch sync${NC}"
echo ""
echo -e "${BLUE}Optional dependencies:${NC}"
echo ""

# Check for AUR helpers (Arch Linux)
if command -v paru &>/dev/null; then
    echo -e "  ${GREEN}✅${NC} paru - ${GREEN}AUR support enabled${NC}"
elif command -v yay &>/dev/null; then
    echo -e "  ${GREEN}✅${NC} yay - ${GREEN}AUR support enabled${NC}"
elif [ -f /etc/arch-release ]; then
    echo -e "  ${YELLOW}○${NC} No AUR helper - ${YELLOW}Install paru:${NC}"
    echo -e "     ${GRAY}paru -S paru${NC}"
fi

# Check for Flatpak
if command -v flatpak &>/dev/null; then
    echo -e "  ${GREEN}✅${NC} flatpak - ${GREEN}Flatpak support enabled${NC}"
else
    echo -e "  ${YELLOW}○${NC} flatpak not found - ${YELLOW}Install:${NC}"
    if [ -f /etc/arch-release ]; then
        echo -e "     ${GRAY}sudo pacman -S flatpak${NC}"
    elif [ -f /etc/debian_version ]; then
        echo -e "     ${GRAY}sudo apt install flatpak${NC}"
    elif [ -f /etc/fedora-release ]; then
        echo -e "     ${GRAY}sudo dnf install flatpak${NC}"
    fi
fi

echo ""
echo -e "${BLUE}Documentation:${NC} ${GREEN}https://nixval.github.io/declarch/${NC}"
echo ""
