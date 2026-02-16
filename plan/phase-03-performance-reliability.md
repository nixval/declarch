# Phase 03 - Performance & Reliability

## Objective
Meningkatkan konsistensi performa dan reliability pada operasi kritikal.

## TODO
- [x] Definisikan benchmark mini untuk path panas:
  - [x] sync transaction path
  - [x] search multi-backend
  - [x] list installed snapshot
- [x] Standardisasi timeout/retry policy lintas module.
- [x] Review penggunaan thread/rayon untuk menghindari overhead tidak perlu.
- [x] Tingkatkan observability ringan (durasi operation-level di mode verbose).
- [x] Evaluasi bottleneck I/O state read/write dan lock contention.

## Baseline Snapshot

- Timestamp (UTC): `2026-02-16T00:22:44Z`
- Harness: `scripts/perf_baseline.sh`
- Current baseline:
  - full test suite proxy: `elapsed=2.788s`
  - sync transaction path (unit proxy): `elapsed=1.916s`
  - search multi-backend selection (unit proxy): `elapsed=1.906s`
  - state sanitize/list snapshot path (unit proxy): `elapsed=1.926s`

- Timestamp (UTC): `2026-02-16T00:30:53Z`
- Harness: `scripts/perf_baseline.sh`
- Post-change snapshot:
  - full test suite proxy: `elapsed=0.385s`
  - sync transaction path (unit proxy): `elapsed=0.271s`
  - search multi-backend selection (unit proxy): `elapsed=0.284s`
  - state sanitize/list snapshot path (unit proxy): `elapsed=0.296s`
  - Note: angka dipengaruhi warm cache/build; dipakai sebagai trend indicator, bukan benchmark absolut lintas mesin.

## Progress Log

- 2026-02-16
  - added reusable baseline script: `scripts/perf_baseline.sh`.
  - captured initial timing snapshot for hot-path proxy scenarios.
  - centralized runtime timeout/retry constants in `constants/common.rs`.
  - wired standardized constants into `generic`, `search`, `hooks`, and `sync/executor`.
  - added backend-level elapsed-time telemetry in `search` when `--verbose` is enabled.
  - reviewed thread model:
    - `search` tetap memakai one-thread-per-backend karena workload dominan I/O eksternal dan jumlah backend kecil.
    - `sync executor` sekarang menggunakan jalur sequential saat jumlah backend `<= 1` untuk menghindari overhead Rayon scheduling.
  - reviewed state I/O and lock contention:
    - `sync --dry-run` tidak lagi menahan lock sepanjang command; sekarang hanya melakukan lock probe lalu release segera.
    - lock contention berkurang untuk run dry-run panjang, tanpa mengubah safety path non-dry-run.
  - reliability check setelah perubahan:
    - `cargo fmt --check`: pass
    - `cargo test --all-targets`: pass

## Exit Criteria
- Ada metrik before/after yang menunjukkan perbaikan.
- Tidak ada regression reliability di operasi sync/search.
