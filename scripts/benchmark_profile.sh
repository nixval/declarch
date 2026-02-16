#!/usr/bin/env bash
set -euo pipefail

BIN="${BIN:-target/debug/declarch}"
ITERATIONS="${ITERATIONS:-8}"
WARMUP="${WARMUP:-2}"
FORMAT="${FORMAT:-markdown}" # markdown | json | both
OUT="${OUT:-}"
OUT_JSON="${OUT_JSON:-plan/test/benchmark-profile.json}"

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
if [[ -z "$OUT" ]]; then
  if [[ "$FORMAT" == "json" ]]; then
    OUT="plan/test/benchmark-profile.json"
  else
    OUT="plan/test/benchmark-profile.md"
  fi
fi
mkdir -p "$(dirname "$OUT")" "$(dirname "$OUT_JSON")"

results_tsv="$(mktemp)"

json_escape() {
  local s="$1"
  s="${s//\\/\\\\}"
  s="${s//\"/\\\"}"
  s="${s//$'\n'/\\n}"
  s="${s//$'\r'/\\r}"
  s="${s//$'\t'/\\t}"
  printf '%s' "$s"
}

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

  printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\n' \
    "$name" "$cmd" "$errors" "$min" "$avg" "$p95" "$max" >> "$results_tsv"

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

write_markdown() {
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
    while IFS=$'\t' read -r name cmd errors min avg p95 max; do
      printf '| %s | `%s` | %s | %s | %s | %s | %s |\n' \
        "$name" "$cmd" "$errors" "$min" "$avg" "$p95" "$max"
    done < "$results_tsv"
  } >> "$OUT"
}

write_json() {
  local target="$1"
  {
    echo "{"
    echo "  \"timestamp_utc\": \"$(json_escape "$timestamp")\","
    echo "  \"binary\": \"$(json_escape "$BIN")\","
    echo "  \"iterations\": $ITERATIONS,"
    echo "  \"warmup\": $WARMUP,"
    echo "  \"temp_root\": \"$(json_escape "$TMP_ROOT")\","
    echo "  \"cases\": ["
    local first=1
    while IFS=$'\t' read -r name cmd errors min avg p95 max; do
      if [[ $first -eq 0 ]]; then
        echo ","
      fi
      first=0
      printf '    {"name":"%s","command":"%s","errors":%s,"min_ms":%s,"avg_ms":%s,"p95_ms":%s,"max_ms":%s}' \
        "$(json_escape "$name")" \
        "$(json_escape "$cmd")" \
        "$errors" "$min" "$avg" "$p95" "$max"
    done < "$results_tsv"
    echo
    echo "  ]"
    echo "}"
  } > "$target"
}

case "$FORMAT" in
  markdown)
    write_markdown
    echo "Benchmark profile written to $OUT"
    ;;
  json)
    write_json "$OUT"
    echo "Benchmark profile written to $OUT"
    ;;
  both)
    write_markdown
    write_json "$OUT_JSON"
    echo "Benchmark profiles written to $OUT and $OUT_JSON"
    ;;
  *)
    echo "Unsupported FORMAT='$FORMAT'. Use: markdown | json | both"
    rm -f "$results_tsv"
    exit 1
    ;;
esac

rm -f "$results_tsv"
