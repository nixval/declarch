#!/bin/bash
# Quick install declarch
set -e

VERSION="0.5.2"
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
curl -fSL "$URL" -o /tmp/declarch.tar.gz
tar xzf /tmp/declarch.tar.gz

if [ ! -f "declarch" ]; then
    echo "Error: Failed to extract declarch binary"
    exit 1
fi

echo "Installing to /usr/local/bin/..."
sudo install -m 755 declarch /usr/local/bin/

echo "✓ Installed declarch $(/usr/local/bin/declarch --version)"

# Cleanup downloaded files
rm -f declarch dcl /tmp/declarch.tar.gz
echo "✓ Cleanup complete"
