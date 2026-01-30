#!/bin/bash
# Quick install declarch
set -e

VERSION="0.5.1"
REPO="nixval/declarch"
ARCH=$(uname -m)

# Map architecture
case "$ARCH" in
    x86_64)  BINARY="declarch-x86_64-unknown-linux-gnu" ;;
    aarch64) BINARY="declarch-aarch64-unknown-linux-gnu" ;;
    *)
        echo "Error: Unsupported architecture: $ARCH"
        exit 1
        ;;
esac

URL="https://github.com/${REPO}/releases/download/v${VERSION}/${BINARY}.tar.gz"

echo "Downloading declarch ${VERSION} (${ARCH})..."
curl -sSL "$URL" | tar xz

echo "Installing to /usr/local/bin/..."
sudo install -m 755 declarch /usr/local/bin/

echo "✓ Installed declarch $(declarch --version)"
echo "Remove downloaded files? (y/n)"
read -r answer
if [[ "$answer" =~ ^[Yy]$ ]]; then
    rm -f declarch dcl
    echo "✓ Cleanup complete"
fi
