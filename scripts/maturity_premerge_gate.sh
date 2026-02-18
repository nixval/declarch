#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

echo "[1/5] cargo fmt --check"
cargo fmt --check

echo "[2/5] cargo test --all-targets --quiet"
RUSTC_WRAPPER= cargo test --all-targets --quiet

echo "[3/5] cargo clippy --all-targets --all-features -- -D warnings"
RUSTC_WRAPPER= cargo clippy --all-targets --all-features -- -D warnings

echo "[4/5] release consistency guard (non-strict)"
scripts/check_release_consistency.sh

echo "[5/5] machine contract examples present"
for f in docs/contracts/v1/info.json docs/contracts/v1/lint.json docs/contracts/v1/search.json docs/contracts/v1/sync-dry-run.json; do
  if [[ ! -f "$f" ]]; then
    echo "ERROR: missing contract example: $f"
    exit 1
  fi
done

echo "Maturity pre-merge gate passed."
