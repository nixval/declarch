# Phase 03 - Performance & Reliability

## Objective
Meningkatkan konsistensi performa dan reliability pada operasi kritikal.

## TODO
- [ ] Definisikan benchmark mini untuk path panas:
  - [ ] sync transaction path
  - [ ] search multi-backend
  - [ ] list installed snapshot
- [ ] Standardisasi timeout/retry policy lintas module.
- [ ] Review penggunaan thread/rayon untuk menghindari overhead tidak perlu.
- [ ] Tingkatkan observability ringan (durasi operation-level di mode verbose).
- [ ] Evaluasi bottleneck I/O state read/write dan lock contention.

## Exit Criteria
- Ada metrik before/after yang menunjukkan perbaikan.
- Tidak ada regression reliability di operasi sync/search.
