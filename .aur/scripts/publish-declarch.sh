#!/usr/bin/env bash
set -euo pipefail

# Publish source package (declarch) to AUR.
# Usage:
#   ./.aur/scripts/publish-declarch.sh [version]
# Example:
#   ./.aur/scripts/publish-declarch.sh 0.8.1

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "${SCRIPT_DIR}/../.." && pwd)"

PKG_NAME="declarch"
AUR_REMOTE="aur@aur.archlinux.org:${PKG_NAME}.git"
TEMPLATE_PATH="${REPO_ROOT}/.aur/templates/PKGBUILD"
WORK_DIR="${REPO_ROOT}/.aur/work/${PKG_NAME}"

VERSION="${1:-}"
if [[ -z "${VERSION}" ]]; then
  VERSION="$(sed -n 's/^version = "\(.*\)"/\1/p' "${REPO_ROOT}/Cargo.toml" | head -1)"
fi

if [[ -z "${VERSION}" ]]; then
  echo -e "${RED}Error: version not provided and not found in Cargo.toml${NC}"
  exit 1
fi

if [[ -x "${REPO_ROOT}/scripts/check_release_consistency.sh" ]]; then
  echo -e "${YELLOW}[preflight] Checking release consistency...${NC}"
  "${REPO_ROOT}/scripts/check_release_consistency.sh" --tag "v${VERSION}" --strict
fi

require_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo -e "${RED}Error: required command not found: $1${NC}"
    exit 1
  fi
}

for cmd in curl sha256sum sed git makepkg mktemp; do
  require_cmd "$cmd"
done

if [[ ! -f "${TEMPLATE_PATH}" ]]; then
  echo -e "${RED}Error: template not found: ${TEMPLATE_PATH}${NC}"
  exit 1
fi

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}  Publish ${PKG_NAME} v${VERSION}${NC}"
echo -e "${BLUE}========================================${NC}"

SOURCE_URL="https://github.com/nixval/declarch/archive/refs/tags/v${VERSION}.tar.gz"

echo -e "${YELLOW}[1/5] Download source tarball for checksum...${NC}"
TMP_TARBALL="$(mktemp)"
trap 'rm -f "${TMP_TARBALL}"' EXIT
curl -fL --retry 3 --retry-delay 2 -o "${TMP_TARBALL}" "${SOURCE_URL}"
SHA256="$(sha256sum "${TMP_TARBALL}" | awk '{print $1}')"
echo "sha256=${SHA256}"

echo -e "${YELLOW}[2/5] Regenerate PKGBUILD in workdir...${NC}"
mkdir -p "${WORK_DIR}"
sed -e "s/^pkgver=.*/pkgver=${VERSION}/" \
    -e "s/^sha256sums=.*/sha256sums=('${SHA256}')/" \
    "${TEMPLATE_PATH}" > "${WORK_DIR}/PKGBUILD"

echo -e "${YELLOW}[3/5] Generate .SRCINFO...${NC}"
(
  cd "${WORK_DIR}"
  makepkg --printsrcinfo > .SRCINFO
)

echo -e "${YELLOW}[4/5] Clone AUR repo and apply update...${NC}"
TMP_REPO="$(mktemp -d)"
trap 'rm -f "${TMP_TARBALL}"; rm -rf "${TMP_REPO}"' EXIT

git clone "${AUR_REMOTE}" "${TMP_REPO}"
cp "${WORK_DIR}/PKGBUILD" "${TMP_REPO}/PKGBUILD"
cp "${WORK_DIR}/.SRCINFO" "${TMP_REPO}/.SRCINFO"

(
  cd "${TMP_REPO}"
  git add PKGBUILD .SRCINFO
  if git diff --cached --quiet; then
    echo -e "${YELLOW}No changes to push for ${PKG_NAME}.${NC}"
    exit 0
  fi
  git commit -m "${PKG_NAME}: update to ${VERSION}"
  echo -e "${YELLOW}[5/5] Push to AUR...${NC}"
  git push origin HEAD:master
)

echo -e "${GREEN}Done: ${PKG_NAME} v${VERSION} published to AUR.${NC}"
