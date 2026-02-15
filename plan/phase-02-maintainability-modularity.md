# Phase 02 - Maintainability & Modularity

## Objective
Menurunkan kompleksitas file inti dan meningkatkan keterbacaan perubahan.

## TODO
- [ ] Pecah file hotspot menjadi submodule:
  - [ ] `backends/user_parser.rs`
  - [ ] `backends/generic.rs`
  - [ ] `commands/sync/mod.rs`
- [ ] Rapikan dispatcher:
  - [ ] ekstrak handler per command group
  - [ ] kurangi nested branching
- [ ] Kurangi type complexity via type alias/struct result objects.
- [ ] Merge logic yang redundant dan hapus dead code terverifikasi.
- [ ] Enforce style consistency agar clippy/fmt stabil.

## Exit Criteria
- File hotspot utama turun kompleksitasnya.
- Behavior tetap konsisten (test hijau).
