#!/usr/bin/env bash
#
# Install declarch via curl or wget from GitHub releases
# Usage: curl -fsSL https://raw.githubusercontent.com/nixval/declarch/main/install-remote.sh | sh
# Or: wget -qO- https://raw.githubusercontent.com/nixval/declarch/main/install-remote.sh | sh
#

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
REPO="nixval/declarch"
BINARY_NAME="declarch"
INSTALL_DIR="/usr/local/bin"
VERSION="${1:-latest}"

# Helper functions
info() {
    echo -e "${BLUE}ℹ${NC} $1"
}

success() {
    echo -e "${GREEN}✓${NC} $1"
}

warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

error() {
    echo -e "${RED}✗${NC} $1"
}

# Detect download method
detect_download_method() {
    if command -v curl &> /dev/null; then
        echo "curl"
    elif command -v wget &> /dev/null; then
        echo "wget"
    else
        error "Neither curl nor wget found. Please install one of them."
        exit 1
    fi
}

# Get latest version from GitHub
get_latest_version() {
    if [[ "$VERSION" == "latest" ]]; then
        if command -v git &> /dev/null; then
            git ls-remote --tags --refs "https://github.com/${REPO}.git" \
                | cut -d '/' -f 3 \
                | sed 's/^v//' \
                | sort -V \
                | tail -n 1
        else
            warning "git not found, using version 0.4.0 as fallback"
            echo "0.4.0"
        fi
    else
        echo "$VERSION"
    fi
}

# Download file
download_file() {
    local url="$1"
    local output="$2"
    local method="$3"

    info "Downloading: $url"

    case "$method" in
        curl)
            curl -fSL -o "$output" "$url"
            ;;
        wget)
            wget -O "$output" "$url"
            ;;
    esac
}

# Main installation
main() {
    echo -e "${BLUE}"
    echo "╔═══════════════════════════════════════════════════════╗"
    echo "║          declarch Installer                          ║"
    echo "║   Declarative Package Manager for Arch Linux         ║"
    echo "╚═══════════════════════════════════════════════════════╝"
    echo -e "${NC}"

    # Detect download method
    local download_method
    download_method=$(detect_download_method)
    info "Using $download_method for downloads"

    # Get version
    local version
    version=$(get_latest_version)
    info "Installing version: ${version}"

    # Detect architecture
    local arch
    arch=$(uname -m)
    case "$arch" in
        x86_64)
            arch="x86_64-unknown-linux-gnu"
            ;;
        aarch64)
            arch="aarch64-unknown-linux-gnu"
            ;;
        *)
            error "Unsupported architecture: $arch"
            exit 1
            ;;
    esac

    # Build download URL
    local filename="${BINARY_NAME}-${version}-${arch}.tar.gz"
    local download_url="https://github.com/${REPO}/releases/download/v${version}/${filename}"

    # Create temp directory
    local tmp_dir
    tmp_dir=$(mktemp -d)
    trap "rm -rf $tmp_dir" EXIT

    # Download binary
    local archive_path="${tmp_dir}/${filename}"
    if ! download_file "$download_url" "$archive_path" "$download_method"; then
        error "Failed to download ${filename}"
        error "Please check: https://github.com/${REPO}/releases"
        exit 1
    fi

    # Extract archive
    info "Extracting archive..."
    cd "$tmp_dir"
    tar -xzf "$filename"

    # Check if binary exists
    if [[ ! -f "${BINARY_NAME}" ]]; then
        error "Binary not found in archive"
        exit 1
    fi

    # Install binary
    info "Installing ${BINARY_NAME} to ${INSTALL_DIR}..."

    # Check if we can write to INSTALL_DIR
    if [[ ! -w "$INSTALL_DIR" ]]; then
        warning "Need sudo permissions to install to ${INSTALL_DIR}"
        sudo install -m 755 "${BINARY_NAME}" "${INSTALL_DIR}/${BINARY_NAME}"
    else
        install -m 755 "${BINARY_NAME}" "${INSTALL_DIR}/${BINARY_NAME}"
    fi

    success "Installation complete!"
    echo ""
    info "Binary installed to: ${INSTALL_DIR}/${BINARY_NAME}"
    echo ""
    info "Quick Start:"
    echo "  1. Initialize declarch:"
    echo "     ${GREEN}declarch init${NC}"
    echo ""
    echo "  2. Sync packages:"
    echo "     ${GREEN}declarch sync${NC}"
    echo ""
    echo "  3. Get help:"
    echo "     ${GREEN}declarch --help${NC}"
    echo ""
    info "Documentation: https://github.com/${REPO}"
}

# Run main function
main "$@"
