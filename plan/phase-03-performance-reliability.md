# Phase 03 - Performance & Reliability

## Objective
Meningkatkan konsistensi performa dan reliability pada operasi kritikal.

## TODO
- [x] Definisikan benchmark mini untuk path panas:
  - [x] sync transaction path
  - [x] search multi-backend
  - [x] list installed snapshot
- [x] Standardisasi timeout/retry policy lintas module.
- [ ] Review penggunaan thread/rayon untuk menghindari overhead tidak perlu.
- [ ] Tingkatkan observability ringan (durasi operation-level di mode verbose).
- [ ] Evaluasi bottleneck I/O state read/write dan lock contention.

## Baseline Snapshot

- Timestamp (UTC): `2026-02-16T00:22:44Z`
- Harness: `scripts/perf_baseline.sh`
- Current baseline:
  - full test suite proxy: `elapsed=2.788s`
  - sync transaction path (unit proxy): `elapsed=1.916s`
  - search multi-backend selection (unit proxy): `elapsed=1.906s`
  - state sanitize/list snapshot path (unit proxy): `elapsed=1.926s`

## Progress Log

- 2026-02-16
  - added reusable baseline script: `scripts/perf_baseline.sh`.
  - captured initial timing snapshot for hot-path proxy scenarios.
  - centralized runtime timeout/retry constants in `constants/common.rs`.
  - wired standardized constants into `generic`, `search`, `hooks`, and `sync/executor`.

## Exit Criteria
- Ada metrik before/after yang menunjukkan perbaikan.
- Tidak ada regression reliability di operasi sync/search.
