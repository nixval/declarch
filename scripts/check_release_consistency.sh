#!/usr/bin/env bash
set -euo pipefail

# Release consistency guard.
# Checks:
# 1) Cargo.toml version
# 2) Optional release tag (must match Cargo version)
# 3) AUR template PKGBUILD version/source pattern
# 4) Optional AUR remote package version (aurweb RPC)
#
# Usage:
#   scripts/check_release_consistency.sh
#   scripts/check_release_consistency.sh --tag v0.8.1
#   scripts/check_release_consistency.sh --tag 0.8.1 --strict --check-aur-remote

STRICT=0
CHECK_AUR_REMOTE=0
TAG_INPUT=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --tag)
      TAG_INPUT="${2:-}"
      shift 2
      ;;
    --strict)
      STRICT=1
      shift
      ;;
    --check-aur-remote)
      CHECK_AUR_REMOTE=1
      shift
      ;;
    *)
      echo "Unknown argument: $1"
      exit 2
      ;;
  esac
done

fail_or_warn() {
  local msg="$1"
  if [[ "$STRICT" -eq 1 ]]; then
    echo "ERROR: ${msg}"
    exit 1
  fi
  echo "WARN: ${msg}"
}

cargo_version="$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)"
if [[ -z "${cargo_version}" ]]; then
  echo "ERROR: unable to parse version from Cargo.toml"
  exit 1
fi
echo "Cargo version: ${cargo_version}"

if [[ -n "${TAG_INPUT}" ]]; then
  normalized_tag="${TAG_INPUT#v}"
  echo "Tag input: ${TAG_INPUT} (normalized: ${normalized_tag})"
  if [[ "${normalized_tag}" != "${cargo_version}" ]]; then
    fail_or_warn "tag version (${normalized_tag}) does not match Cargo.toml (${cargo_version})"
  fi
fi

template_path=".aur/templates/PKGBUILD"
if [[ ! -f "${template_path}" ]]; then
  fail_or_warn "missing ${template_path}"
else
  template_pkgver="$(sed -n 's/^pkgver=\(.*\)/\1/p' "${template_path}" | head -1)"
  if [[ -z "${template_pkgver}" ]]; then
    fail_or_warn "unable to parse pkgver from ${template_path}"
  else
    echo "AUR template pkgver: ${template_pkgver}"
    if [[ "${template_pkgver}" != "${cargo_version}" ]]; then
      fail_or_warn "AUR template pkgver (${template_pkgver}) does not match Cargo.toml (${cargo_version})"
    fi
  fi

  if ! grep -Eq 'archive/refs/tags/v\$pkgver\.tar\.gz' "${template_path}"; then
    fail_or_warn "AUR template source does not use refs/tags/v\$pkgver.tar.gz pattern"
  fi
fi

if [[ "${CHECK_AUR_REMOTE}" -eq 1 ]]; then
  if ! command -v curl >/dev/null 2>&1; then
    fail_or_warn "curl not found; cannot check AUR remote version"
  else
    rpc_json="$(curl -fsSL "https://aur.archlinux.org/rpc/?v=5&type=info&arg[]=declarch" || true)"
    remote_ver="$(printf '%s' "${rpc_json}" | sed -n 's/.*"Version":"\([^"]*\)".*/\1/p' | head -1)"
    if [[ -z "${remote_ver}" ]]; then
      fail_or_warn "unable to parse declarch version from AUR RPC response"
    else
      remote_pkgver="${remote_ver%%-*}"
      echo "AUR remote version: ${remote_ver} (pkgver: ${remote_pkgver})"
      if [[ "${remote_pkgver}" == "${cargo_version}" ]]; then
        echo "AUR remote already matches Cargo version."
      else
        echo "INFO: AUR remote differs from Cargo version (expected before AUR publish)."
      fi
    fi
  fi
fi

echo "Release consistency checks complete."
