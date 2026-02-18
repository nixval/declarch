#!/usr/bin/env bash
set -euo pipefail

# Publish both AUR packages with preflight checks.
# Usage:
#   ./.aur/scripts/publish-aur-all.sh [version]
# Example:
#   ./.aur/scripts/publish-aur-all.sh 0.8.2

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "${SCRIPT_DIR}/../.." && pwd)"

VERSION="${1:-}"
if [[ -z "${VERSION}" ]]; then
  VERSION="$(sed -n 's/^version = "\(.*\)"/\1/p' "${REPO_ROOT}/Cargo.toml" | head -1)"
fi

if [[ -z "${VERSION}" ]]; then
  echo -e "${RED}Error: version not provided and not found in Cargo.toml${NC}"
  exit 1
fi

require_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo -e "${RED}Error: required command not found: $1${NC}"
    exit 1
  fi
}

for cmd in git curl makepkg sha256sum; do
  require_cmd "$cmd"
done

if [[ ! -x "${SCRIPT_DIR}/publish-declarch.sh" ]]; then
  echo -e "${RED}Missing script: ${SCRIPT_DIR}/publish-declarch.sh${NC}"
  exit 1
fi
if [[ ! -x "${SCRIPT_DIR}/publish-declarch-bin.sh" ]]; then
  echo -e "${RED}Missing script: ${SCRIPT_DIR}/publish-declarch-bin.sh${NC}"
  exit 1
fi

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}  AUR Publish All v${VERSION}${NC}"
echo -e "${BLUE}========================================${NC}"

echo -e "${YELLOW}[1/4] Preflight: check GitHub release tag...${NC}"
if ! git ls-remote --tags https://github.com/nixval/declarch.git "refs/tags/v${VERSION}" | grep -q "v${VERSION}"; then
  echo -e "${RED}Tag v${VERSION} not found on GitHub remote.${NC}"
  echo "Push tag first: git push origin v${VERSION}"
  exit 1
fi
echo -e "${GREEN}OK: tag v${VERSION} found${NC}"

echo -e "${YELLOW}[2/4] Publish declarch (source)...${NC}"
"${SCRIPT_DIR}/publish-declarch.sh" "${VERSION}"

echo -e "${YELLOW}[3/4] Publish declarch-bin (binary)...${NC}"
"${SCRIPT_DIR}/publish-declarch-bin.sh" "${VERSION}"

echo -e "${YELLOW}[4/4] Done. Quick verify:${NC}"
echo "  https://aur.archlinux.org/packages/declarch"
echo "  https://aur.archlinux.org/packages/declarch-bin"
echo -e "${GREEN}All publish steps completed.${NC}"
