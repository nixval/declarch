#!/usr/bin/env bash
set -euo pipefail

fail=0

echo "Checking identity literals..."

# 1) No hardcoded DECLARCH_* env keys in src string literals.
if rg -n '"DECLARCH_[A-Z0-9_]+' src >/tmp/identity-literals-env.txt 2>/dev/null; then
  echo "Found hardcoded DECLARCH_* env key literals in src:"
  cat /tmp/identity-literals-env.txt
  fail=1
fi

# 2) Repo slug should come from project_identity.rs only.
if rg -n '"nixval/declarch' src -g '!src/project_identity.rs' >/tmp/identity-literals-repo.txt 2>/dev/null; then
  echo "Found hardcoded repo slug outside src/project_identity.rs:"
  cat /tmp/identity-literals-repo.txt
  fail=1
fi

# 3) User-Agent suffix should come from identity constants.
if rg -n '"declarch-cli"' src -g '!src/project_identity.rs' >/tmp/identity-literals-ua.txt 2>/dev/null; then
  echo "Found hardcoded user-agent literal outside src/project_identity.rs:"
  cat /tmp/identity-literals-ua.txt
  fail=1
fi

if [[ "$fail" -ne 0 ]]; then
  exit 1
fi

echo "Identity literal checks passed."
