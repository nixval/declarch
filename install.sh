#!/bin/bash
# Quick install declarch
set -e

VERSION="0.8.0"
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

# Check for existing declarch installations
EXISTING_DECLARCH=$(which -a declarch 2>/dev/null || true)
if [ -n "$EXISTING_DECLARCH" ]; then
    echo "⚠️  Found existing declarch installations:"
    echo "$EXISTING_DECLARCH" | while read -r path; do
        if [ -n "$path" ]; then
            echo "  - $path"
        fi
    done
    echo ""
    echo "This may cause conflicts. The new version will be installed to /usr/local/bin/declarch"
    echo "After installation, you may want to remove old versions:"
    echo "  sudo rm -f /usr/bin/declarch /usr/sbin/declarch /bin/declarch /sbin/declarch"
    echo "  rm -f ~/.local/bin/declarch"
    echo ""
fi

echo "Installing to /usr/local/bin/..."
sudo install -m 755 declarch /usr/local/bin/

# Create short alias command (decl -> declarch) if possible.
# Keep existing user-managed decl binary untouched.
if [ -e /usr/local/bin/decl ] && [ ! -L /usr/local/bin/decl ]; then
    echo "⚠️  Skipping alias creation: /usr/local/bin/decl already exists and is not a symlink."
else
    sudo ln -sfn /usr/local/bin/declarch /usr/local/bin/decl
fi

INSTALLED_VERSION=$(/usr/local/bin/declarch --version)
echo "✓ Installed $INSTALLED_VERSION to /usr/local/bin/declarch"
if [ -x /usr/local/bin/decl ]; then
    echo "✓ Alias installed: /usr/local/bin/decl -> /usr/local/bin/declarch"
fi

# Verify it's in PATH and accessible
if ! command -v declarch &>/dev/null; then
    echo "⚠️  Warning: /usr/local/bin is not in your PATH"
    echo "Add it to your PATH or use: /usr/local/bin/declarch"
fi

# Cleanup downloaded files
rm -f declarch decl dcl /tmp/declarch.tar.gz
echo "✓ Cleanup complete"
