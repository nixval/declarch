#!/usr/bin/env bash
set -euo pipefail

BIN="${BIN:-target/debug/declarch}"
ITERATIONS="${ITERATIONS:-8}"
WARMUP="${WARMUP:-2}"
OUT="${OUT:-plan/test/benchmark-profile.md}"

if [[ ! -x "$BIN" ]]; then
  echo "Binary not found: $BIN"
  echo "Build first: cargo build"
  exit 1
fi

TMP_ROOT="$(mktemp -d)"
export HOME="$TMP_ROOT/home"
export XDG_CONFIG_HOME="$TMP_ROOT/config"
export XDG_STATE_HOME="$TMP_ROOT/state"
export XDG_CACHE_HOME="$TMP_ROOT/cache"
mkdir -p "$HOME" "$XDG_CONFIG_HOME" "$XDG_STATE_HOME" "$XDG_CACHE_HOME"

# Prepare minimal config state for command paths that expect initialized setup.
"$BIN" init >/dev/null 2>&1 || true

timestamp="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
mkdir -p "$(dirname "$OUT")"

cat > "$OUT" <<'MARKDOWN'
# Benchmark Profile
MARKDOWN

{
  echo
  echo "- Timestamp (UTC): \`$timestamp\`"
  echo "- Binary: \`$BIN\`"
  echo "- Iterations: \`$ITERATIONS\`"
  echo "- Warmup: \`$WARMUP\`"
  echo "- Temp root: \`$TMP_ROOT\`"
  echo
  echo "| Case | Command | Errors | Min ms | Avg ms | P95 ms | Max ms |"
  echo "|---|---|---:|---:|---:|---:|---:|"
} >> "$OUT"

run_case() {
  local name="$1"
  local cmd="$2"

  local times_file
  times_file="$(mktemp)"
  local errors=0

  # warmup
  for _ in $(seq 1 "$WARMUP"); do
    set +e
    bash -lc "$BIN $cmd" >/dev/null 2>&1
    set -e
  done

  for _ in $(seq 1 "$ITERATIONS"); do
    local start_ns end_ns elapsed_ms
    start_ns="$(date +%s%N)"
    set +e
    bash -lc "$BIN $cmd" >/dev/null 2>&1
    local code=$?
    set -e
    end_ns="$(date +%s%N)"

    elapsed_ms=$(( (end_ns - start_ns) / 1000000 ))
    echo "$elapsed_ms" >> "$times_file"

    if [[ $code -ne 0 ]]; then
      errors=$((errors + 1))
    fi
  done

  local min avg p95 max
  min="$(sort -n "$times_file" | head -n1)"
  max="$(sort -n "$times_file" | tail -n1)"
  avg="$(awk '{s+=$1} END {if (NR==0) print 0; else printf "%.1f", s/NR}' "$times_file")"
  p95="$(sort -n "$times_file" | awk 'BEGIN{n=0} {a[++n]=$1} END{if(n==0){print 0; exit} idx=int((n*95+99)/100); if(idx<1) idx=1; if(idx>n) idx=n; print a[idx]}')"

  printf '| %s | `%s` | %s | %s | %s | %s | %s |\n' \
    "$name" "$cmd" "$errors" "$min" "$avg" "$p95" "$max" >> "$OUT"

  rm -f "$times_file"
}

run_case "help" "--help"
run_case "version" "--version"
run_case "init list backends" "init --list backends"
run_case "info doctor" "info --doctor"
run_case "lint validate" "lint --mode validate"
run_case "sync preview" "sync preview --yes"
run_case "search no-backend" "search bat --backends doesnotexist --limit 1"
run_case "sync preview machine" "--format json --output-version v1 sync preview"

echo "Benchmark profile written to $OUT"
