# Plan Index

Dokumen ini adalah index TODO eksekusi roadmap quality.

## Phases
- `plan/phase-00-baseline.md`
- `plan/phase-01-security-correctness.md`
- `plan/phase-02-maintainability-modularity.md`
- `plan/phase-03-performance-reliability.md`
- `plan/phase-04-beginner-experience.md`
- `plan/phase-05-release-guardrails.md`

## Supporting Docs
- `plan/commit-strategy.md`
- `plan/risk-register.md`
- `plan/rfc-flag-subcommand-surface-simplification.md`

## Execution Rules
- Setiap TODO yang selesai harus diikuti commit kecil.
- Jangan gabung banyak concern dalam satu commit.
- Jalankan quality gate minimal sebelum commit:
  - `cargo fmt --check`
  - `cargo clippy --all-targets -- -D warnings` (saat phase lint-hardening)
  - `cargo test --all-targets`
