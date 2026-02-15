# Phase 00 - Baseline & Inventory

## Objective
Membuat baseline objektif supaya progres bisa diukur.

## TODO
- [x] Rekam baseline metrik:
  - [x] jumlah file Rust
  - [x] total LOC
  - [x] file >700 LOC
  - [x] waktu `cargo test --all-targets`
  - [x] jumlah warning/error clippy strict
- [x] Buat daftar hotspot modul:
  - [x] dispatcher
  - [x] sync orchestration
  - [x] backend parser/manager
  - [x] state I/O
- [x] Tetapkan urutan eksekusi refactor (berdasarkan risiko dan dampak).
- [x] Lock acceptance criteria tiap phase sebelum coding besar.

## Baseline Snapshot

- Timestamp (UTC): `2026-02-15T17:22:37Z`
- Rust source files:
  - `src`: 98 file
  - `tests`: 6 file
  - test blocks in source/tests (`mod tests` or `#[cfg(test)]`): 85
- Total LOC:
  - `src`: 23,410 LOC
- Top file hotspots (LOC):
  - `src/backends/user_parser.rs`: 1361
  - `src/backends/generic.rs`: 1219
  - `src/commands/sync/mod.rs`: 1093
  - `src/commands/init/backend.rs`: 915
  - `src/commands/lint.rs`: 761
- LOC by domain folder:
  - `commands`: 9089
  - `backends`: 4256
  - `config`: 3658
  - `utils`: 1476
  - `cli`: 1239
  - `core`: 958
  - `state`: 798
- Test performance baseline:
  - `cargo test --all-targets --quiet`: pass
  - elapsed wall time: `4.395s`
- Clippy strict baseline:
  - Command: `cargo clippy --all-targets -- -D warnings`
  - Exit code: `101` (failed)
  - `error:` lines in output: 25
  - Key compile summary in output:
    - `could not compile declarch (lib) due to 22 previous errors`
    - `could not compile declarch (lib test) due to 23 previous errors`
  - Dominant categories:
    - `collapsible_if` / `collapsible_else_if`
    - `needless_return`
    - `type_complexity`
    - `items_after_test_module`
    - `useless_format`

## Hotspot Inventory (Risk-Oriented)

- Dispatcher complexity
  - `src/cli/dispatcher.rs`
  - risk: branching dan command routing menjadi bottleneck perubahan UX
- Sync orchestration
  - `src/commands/sync/mod.rs`
  - `src/commands/sync/executor.rs`
  - risk: flow panjang (lock/config/plan/execute/state/hooks) rawan regression
- Backend parser/manager
  - `src/backends/user_parser.rs`
  - `src/backends/generic.rs`
  - risk: kombinasi parsing + execution + compatibility logic paling kompleks
- State I/O
  - `src/state/io.rs`
  - risk: lock/state integrity/recovery menyentuh correctness kritikal

## Refactor Priority (Phase 01-03)

1. Phase 01: security dan correctness dulu (hardening tanpa refactor besar)
2. Phase 02: modularisasi hotspot, mulai dari backend parser/manager lalu sync orchestration
3. Phase 03: tuning performance/reliability dengan benchmark before/after

## Acceptance Criteria Lock

- Phase 01 accepted jika:
  - bug/security prioritas ditutup,
  - ada test tambahan untuk area kritikal,
  - tidak ada regression pada `cargo test --all-targets`.
- Phase 02 accepted jika:
  - hotspot utama dipecah/dirapikan,
  - kompleksitas file menurun,
  - behavior tetap (test hijau).
- Phase 03 accepted jika:
  - ada metrik before/after pada path panas,
  - reliability sync/search tetap stabil,
  - tidak ada penurunan kualitas lint/test.

## Exit Criteria
- Baseline terdokumentasi dan disepakati.
- Prioritas phase 01-03 final.
