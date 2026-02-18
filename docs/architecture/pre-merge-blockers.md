# Pre-Merge Blockers (Release Readiness)

Gunakan dokumen ini sebagai runbook singkat kapan merge ke `main` harus diblokir meskipun sebagian test hijau.

## Hard blockers

1. `scripts/maturity_premerge_gate.sh` gagal di salah satu step.
2. `cargo clippy --all-targets --all-features -- -D warnings` gagal.
3. `cargo test --all-targets` gagal di test command surface (`sync/search/lint/info`).
4. Contract machine output v1 examples hilang/tidak sinkron (`docs/contracts/v1/*`).
5. Release consistency mismatch (`Cargo.toml` vs `.aur/templates/PKGBUILD` versi).

## Soft blockers (escalate to hard blocker if high-risk)

1. Ada perubahan behavior user-facing tanpa test parity/assertion baru.
2. Ada perubahan modul kritikal (`state/io`, `sync/executor`, `hooks`) tanpa test branch tambahan.
3. Ada warning operasional baru (lock contention, backup recovery, update check) yang belum didokumentasikan.

## Merge decision

- Merge only if semua hard blocker cleared.
- Jika ada soft blocker, butuh approval maintainer + issue follow-up dengan due date.

## Quick commands

```bash
scripts/maturity_premerge_gate.sh
RUSTC_WRAPPER= cargo test --all-targets --quiet
RUSTC_WRAPPER= cargo clippy --all-targets --all-features -- -D warnings
```
