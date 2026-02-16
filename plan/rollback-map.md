# Rollback Map

This map tracks high-impact batch commits so rollback can be targeted and low-risk.

## How to rollback safely

1. Identify the smallest commit affecting the regression.
2. Revert with:

```bash
git revert <commit-sha>
```

3. Re-run quality gates:

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
```

## Batch Commit Map

### RFC + Planning bootstrap
- `ac498ec` RFC + plan folder + gitignore plan policy

### Phase 00 baseline
- `8835095` baseline inventory and initial metrics snapshot

### Phase 01 security + correctness
- `686526e` shell escaping correctness fix + tests
- `d6c9d2c` search `--limit` fail-fast parsing
- `dc5c639` private network block fix for 172.16/12
- `08c5b6f` remote URL hardening and malformed-host handling
- `cf71b09` HTTP opt-in via `DECLARCH_ALLOW_INSECURE_HTTP=1`
- `feafbb1` fetch diagnostics improvement

### Phase 02 maintainability + modularity
- `6abd968` dispatcher handler extraction
- `42e377f` backend registry type complexity reduction
- `2d3c1a4` sync preview helper extraction
- `70e48f2` sync policy module extraction
- `77592b5` generic backend command execution extraction
- `3388a62` user parser validation extraction
- `7e7f95b` + `15978ba` clippy cleanup batches
- `bec37cb` phase completion checkpoint with clean gates

### Phase 03 performance + reliability
- `9bf9118` baseline timing harness
- `b77ca26` timeout/retry constants standardization
- `cbadcab` search verbose timing telemetry
- `285cf43` sync lock/rayon overhead reduction

### Phase 04 beginner experience
- `17b26ff` onboarding docs and CLI help/actionable guidance

## Notes

- Revert newer commits first if dependencies exist between batches.
- If multiple commits touch same file, prefer reverting as a sequence and resolving conflicts once.
