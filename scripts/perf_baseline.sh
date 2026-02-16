#!/usr/bin/env bash
set -euo pipefail

run_case() {
  local label="$1"
  local filter="$2"
  local tf="elapsed=0"
  echo "[case] $label"
  TIMEFORMAT='elapsed=%3R'
  local out
  out=$({ time cargo test --quiet "$filter"; } 2>&1)
  echo "$out" | rg "test result: ok\. 1 passed|elapsed=" || true
  echo
}

echo "== declarch perf baseline =="
date -u +"timestamp=%Y-%m-%dT%H:%M:%SZ"

echo "[case] full test suite"
TIMEFORMAT='elapsed=%3R'
out=$({ time cargo test --all-targets --quiet; } 2>&1)
echo "$out" | rg "test result: ok\.|elapsed=" | tail -n 4

echo
run_case "sync transaction path (unit proxy)" "commands::sync::tests::test_apply_backend_package_sources_normalizes_and_dedupes"
run_case "search multi-backend selection (unit proxy)" "commands::search::tests::select_backends_filters_unknown_and_unsupported"
run_case "state sanitize/list snapshot path (unit proxy)" "state::io::tests::sanitize_removes_empty_and_rekeys"
