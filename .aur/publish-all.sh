#!/usr/bin/env bash
#
# Publish all declarch packages to AUR
# Publishes: declarch, declarch-bin, declarch-git
#

set -e

echo "==================================="
echo "Publishing ALL declarch packages"
echo "==================================="
echo ""

# Check which script was called from
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

# Publish declarch
echo ""
echo "1/3: Publishing declarch..."
"$SCRIPT_DIR/publish-declarch.sh" "$@"

echo ""
echo "==================================="
echo ""

# Publish declarch-bin
echo "2/3: Publishing declarch-bin..."
"$SCRIPT_DIR/publish-declarch-bin.sh" "$@"

echo ""
echo "==================================="
echo ""

# Publish declarch-git
echo "3/3: Publishing declarch-git..."
"$SCRIPT_DIR/publish-declarch-git.sh"

echo ""
echo "==================================="
echo "✓ All packages published successfully!"
echo "==================================="
echo ""
echo "Packages published:"
echo "  - declarch (source build)"
echo "  - declarch-bin (pre-built binary)"
echo "  - declarch-git (latest git version)"
