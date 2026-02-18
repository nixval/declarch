# Pre-Merge Maturity Checklist (Target >= 9.5)

Tujuan dokumen ini: menurunkan risiko regressions sebelum merge branch `feat/0.8.2-feature` menjadi release terbaru.

Status legend:
- `[x]` selesai
- `[~]` in progress
- `[ ]` belum

## A. Baseline Quality Gate

- [x] `cargo fmt`
- [x] `RUSTC_WRAPPER= cargo test --all-targets --quiet`
- [x] `RUSTC_WRAPPER= cargo clippy --all-targets --all-features -- -D warnings`
- [x] Lulus pada branch `main` dan branch `feat/0.8.2-feature`

## B. Behavior Parity (No Unexpected Behavior Change)

- [~] Tambah golden/snapshot test untuk command surface utama:
  - [x] parity suite minimum (CLI output-contract validation path)
  - [ ] `sync` (`dry-run`, `prune`, `update`) golden snapshot
  - [ ] `search` (with/without backend filter, local mode) golden snapshot
  - [ ] `lint` (`all`, `validate`, strict mode) golden snapshot
  - [ ] `info` + `info --list` golden snapshot
- [x] Tambah parity assertions untuk pesan error utama yang bersifat user-facing.

## C. Machine Output Contract v1 Hardening

- [x] Update kontrak docs agar mencakup command yang sudah support v1.
- [x] Tambah contoh untuk `info --list` v1 envelope.
- [x] Tambah contoh warning/error envelope untuk `search` dan `sync` dry-run.
- [x] Tambah test validasi struktur envelope (`version/command/ok/data/warnings/errors/meta`).

## D. Module-Level Regression Reinforcement

### D1. Parser stages
- [x] Tambah test untuk `parser/ast_scan.rs`
- [x] Tambah test untuk `parser/semantic_mapping.rs`

### D2. Sync runtime critical paths
- [x] Tambah test untuk `executor/retry.rs`
- [x] Tambah test branch untuk `executor/install_ops.rs`
- [x] Tambah test branch untuk `executor/prune.rs`
- [x] Tambah test branch untuk `planner/filtering.rs`

### D3. Hook safety
- [x] Ekstrak validasi command hook agar testable.
- [x] Tambah test validasi command hook (unsafe chars, sudo, traversal).
- [x] Tambah test dry-run path dan required-hook failure path.

## E. State Reliability

- [x] Test migration edge case (dedup + metadata normalization).
- [x] Tambah test lock contention scenario (write path).
- [x] Tambah test backup-restore fallback berlapis (lebih dari 1 backup file).

## F. Release/Operational Readiness

- [x] Tambah script gate pre-merge terpusat (`scripts/maturity_premerge_gate.sh`).
- [x] Integrasikan script gate ke workflow CI release branch.
- [x] Tambah runbook singkat kapan merge harus diblokir walau test hijau.

## G. Final Merge Criteria (Score Gate)

Minimum untuk klaim >= 9.5:
- [x] Semua item section A wajib hijau.
- [x] Semua item section C wajib hijau.
- [x] Minimal 80% item section D hijau.
- [x] Minimal 1 parity suite pada section B hijau.
- [ ] Tidak ada TODO blocker tersisa di command surface (`sync/search/lint/info`).

## Commands

```bash
cargo fmt
RUSTC_WRAPPER= cargo test --all-targets --quiet
RUSTC_WRAPPER= cargo clippy --all-targets --all-features -- -D warnings
scripts/maturity_premerge_gate.sh
```
