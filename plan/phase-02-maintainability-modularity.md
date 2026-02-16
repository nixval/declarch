# Phase 02 - Maintainability & Modularity

## Objective
Menurunkan kompleksitas file inti dan meningkatkan keterbacaan perubahan.

## TODO
- [ ] Pecah file hotspot menjadi submodule:
  - [ ] `backends/user_parser.rs`
  - [~] `backends/generic.rs` (in progress: command execution/timeouts extracted)
  - [x] `commands/sync/mod.rs`
- [x] Rapikan dispatcher:
  - [x] ekstrak handler per command group
  - [x] kurangi nested branching
- [x] Kurangi type complexity via type alias/struct result objects.
- [ ] Merge logic yang redundant dan hapus dead code terverifikasi.
- [ ] Enforce style consistency agar clippy/fmt stabil.

## Progress Log

- 2026-02-15
  - extracted `cli::dispatcher` command-group handlers (`init/sync/info/search/lint`) to reduce monolithic branching.
  - introduced backend registry type aliases (`BackendSourceMap`, `BackendsWithSources`) to reduce signature complexity.
  - extracted sync preview report construction into helper for clearer `sync::run` flow.
  - moved sync policy/hook-gating logic into `sync/policy.rs` to reduce orchestration file density.
  - extracted generic backend command execution + timeout functions into `backends/generic/command_exec.rs`.

## Exit Criteria
- File hotspot utama turun kompleksitasnya.
- Behavior tetap konsisten (test hijau).
