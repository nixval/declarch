#!/usr/bin/env bash
# Quick install declarch (Linux + macOS)
set -eu
# Keep compatibility when invoked via `... | sh` (dash doesn't support pipefail).
if (set -o pipefail) >/dev/null 2>&1; then
  set -o pipefail
fi

DECLARCH_VERSION="${DECLARCH_VERSION:-latest}"
REPO_SLUG="nixval/declarch"
BIN_NAME="declarch"
BIN_ALIAS="decl"
ASSET_PREFIX="declarch"
STABLE_ID="declarch"
OS="$(uname -s)"
ARCH="$(uname -m)"
TMP_DIR="$(mktemp -d)"
ARCHIVE="${TMP_DIR}/${BIN_NAME}.tar.gz"
CHECKSUMS_FILE="${TMP_DIR}/checksums.txt"

sha256_file() {
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$1" | awk '{print $1}'
    return 0
  fi
  if command -v shasum >/dev/null 2>&1; then
    shasum -a 256 "$1" | awk '{print $1}'
    return 0
  fi
  if command -v openssl >/dev/null 2>&1; then
    openssl dgst -sha256 "$1" | awk '{print $NF}'
    return 0
  fi
  echo "Error: no SHA256 tool found (need sha256sum, shasum, or openssl)." >&2
  exit 1
}

cleanup() {
  rm -rf "${TMP_DIR}"
}
trap cleanup EXIT

case "${OS}" in
  Linux)
    case "${ARCH}" in
      x86_64) TARGET="x86_64-unknown-linux-gnu" ;;
      aarch64|arm64) TARGET="aarch64-unknown-linux-gnu" ;;
      *)
        echo "Error: unsupported Linux architecture '${ARCH}'"
        exit 1
        ;;
    esac
    ;;
  Darwin)
    echo "WARNING: macOS installer path is experimental (alpha)."
    echo "Use on non-production machines first and validate with '${BIN_NAME} info' + '${BIN_NAME} lint'."
    case "${ARCH}" in
      x86_64) TARGET="x86_64-apple-darwin" ;;
      arm64|aarch64) TARGET="aarch64-apple-darwin" ;;
      *)
        echo "Error: unsupported macOS architecture '${ARCH}'"
        exit 1
        ;;
    esac
    ;;
  *)
    echo "Error: unsupported OS '${OS}'. Use install.ps1 on Windows."
    exit 1
    ;;
esac

if [ "${DECLARCH_VERSION}" = "latest" ]; then
  BASE_URL="https://github.com/${REPO_SLUG}/releases/latest/download"
  echo "Downloading ${BIN_NAME} (latest release) for ${TARGET}..."
else
  BASE_URL="https://github.com/${REPO_SLUG}/releases/download/v${DECLARCH_VERSION}"
  echo "Downloading ${BIN_NAME} ${DECLARCH_VERSION} for ${TARGET}..."
fi
ASSET_NAME="${ASSET_PREFIX}-${TARGET}.tar.gz"
URL="${BASE_URL}/${ASSET_NAME}"
CHECKSUMS_URL="${BASE_URL}/checksums.txt"
curl -fsSL "${URL}" -o "${ARCHIVE}"
curl -fsSL "${CHECKSUMS_URL}" -o "${CHECKSUMS_FILE}"
EXPECTED_SHA="$(awk -v asset="${ASSET_NAME}" '$2==asset{print $1; exit}' "${CHECKSUMS_FILE}")"
if [ -z "${EXPECTED_SHA}" ]; then
  echo "Error: checksum entry for ${ASSET_NAME} not found in checksums.txt"
  exit 1
fi
ACTUAL_SHA="$(sha256_file "${ARCHIVE}")"
if [ "${ACTUAL_SHA}" != "${EXPECTED_SHA}" ]; then
  echo "Error: checksum verification failed for ${ASSET_NAME}"
  echo "Expected: ${EXPECTED_SHA}"
  echo "Actual:   ${ACTUAL_SHA}"
  exit 1
fi
echo "Checksum verified: ${ASSET_NAME}"
tar xzf "${ARCHIVE}" -C "${TMP_DIR}"

if [ ! -f "${TMP_DIR}/${BIN_NAME}" ]; then
  echo "Error: failed to extract ${BIN_NAME} binary from release archive"
  exit 1
fi

# Prefer /usr/local/bin when writable (or with sudo), otherwise fallback to user bin.
INSTALL_DIR="/usr/local/bin"
USE_SUDO=""
if [ ! -w "${INSTALL_DIR}" ]; then
  if command -v sudo >/dev/null 2>&1; then
    USE_SUDO="sudo"
  else
    INSTALL_DIR="${HOME}/.local/bin"
    mkdir -p "${INSTALL_DIR}"
  fi
fi

echo "Installing to ${INSTALL_DIR}..."
${USE_SUDO} install -m 755 "${TMP_DIR}/${BIN_NAME}" "${INSTALL_DIR}/${BIN_NAME}"

# Create short alias if safe.
if [ -e "${INSTALL_DIR}/${BIN_ALIAS}" ] && [ ! -L "${INSTALL_DIR}/${BIN_ALIAS}" ]; then
  echo "Skipping alias: ${INSTALL_DIR}/${BIN_ALIAS} exists and is not a symlink."
else
  ${USE_SUDO} ln -sfn "${INSTALL_DIR}/${BIN_NAME}" "${INSTALL_DIR}/${BIN_ALIAS}"
fi

INSTALLED_VERSION="$("${INSTALL_DIR}/${BIN_NAME}" --version)"
echo "Installed ${INSTALLED_VERSION} to ${INSTALL_DIR}/${BIN_NAME}"

# Persist installation channel marker for update guidance (best-effort).
STATE_BASE="${XDG_STATE_HOME:-${HOME}/.local/state}"
MARKER_DIR="${STATE_BASE}/${STABLE_ID}"
MARKER_PATH="${MARKER_DIR}/install-channel.json"
mkdir -p "${MARKER_DIR}" 2>/dev/null || true
cat > "${MARKER_PATH}" <<EOF || true
{"channel":"script","installed_at":"$(date -u +"%Y-%m-%dT%H:%M:%SZ")"}
EOF

if ! command -v "${BIN_NAME}" >/dev/null 2>&1; then
  echo "Note: '${INSTALL_DIR}' is not in PATH for this shell."
  if [ "${INSTALL_DIR}" = "${HOME}/.local/bin" ]; then
    echo "Add this line to your shell profile:"
    echo "  export PATH=\"\$HOME/.local/bin:\$PATH\""
  fi
fi

# Lightweight smoke checks (safe on fresh machines, no config required).
echo "Running smoke checks..."
"${INSTALL_DIR}/${BIN_NAME}" --help >/dev/null
"${INSTALL_DIR}/${BIN_NAME}" info >/dev/null || true
echo "Smoke checks complete."

echo "Install complete."
