# Phase 05 - Release Guardrails & Stabilization

## Objective
Menjaga kualitas tetap stabil setelah refactor/peningkatan selesai.

## TODO
- [x] Finalisasi CI quality gates:
  - [x] fmt check
  - [x] clippy strict
  - [x] tests all targets
- [x] Pastikan changelog discipline untuk setiap perubahan behavior.
- [x] Siapkan release checklist berbasis phase completion.
- [x] Verifikasi rollback path per batch commit.
- [x] Freeze window untuk regression watch sebelum release final.

## Progress Log

- 2026-02-16
  - finalized CI gate commands in `.github/workflows/ci.yml`:
    - `cargo fmt --check`
    - `cargo clippy --all-targets -- -D warnings`
    - `cargo test --all-targets --verbose`
  - added changelog discipline policy in `CONTRIBUTING.md`.
  - hardened `scripts/release.sh`:
    - gate alignment with CI commands.
    - enforce presence of `CHANGELOG.md` `[Unreleased]` section.
    - require at least one bullet under `[Unreleased]`.
  - added release governance docs:
    - `RELEASE_CHECKLIST.md`
    - `plan/rollback-map.md`
  - documented freeze-window process (24h regression watch) in checklist.

## Exit Criteria
- Guardrails release repeatable.
- Risiko regressi pasca-release turun signifikan.
