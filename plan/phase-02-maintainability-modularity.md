# Phase 02 - Maintainability & Modularity

## Objective
Menurunkan kompleksitas file inti dan meningkatkan keterbacaan perubahan.

## TODO
- [ ] Pecah file hotspot menjadi submodule:
  - [x] `backends/user_parser.rs`
  - [x] `backends/generic.rs`
  - [x] `commands/sync/mod.rs`
- [x] Rapikan dispatcher:
  - [x] ekstrak handler per command group
  - [x] kurangi nested branching
- [x] Kurangi type complexity via type alias/struct result objects.
- [x] Merge logic yang redundant dan hapus dead code terverifikasi.
- [x] Enforce style consistency agar clippy/fmt stabil.

## Progress Log

- 2026-02-15
  - extracted `cli::dispatcher` command-group handlers (`init/sync/info/search/lint`) to reduce monolithic branching.
  - introduced backend registry type aliases (`BackendSourceMap`, `BackendsWithSources`) to reduce signature complexity.
  - extracted sync preview report construction into helper for clearer `sync::run` flow.
  - moved sync policy/hook-gating logic into `sync/policy.rs` to reduce orchestration file density.
  - extracted generic backend command execution + timeout functions into `backends/generic/command_exec.rs`.
  - extracted user backend parser validation logic into `backends/user_parser/validation.rs`.
  - applied low-risk clippy cleanups (`generic`, `cache`, `info_reason`, `platform`, `state/io`).
  - clippy `-D warnings` error count reduced from 21 to 15 in current phase checkpoint.
  - completed formatter-safe clippy cleanup batch across `edit/init/search/sync/platform/remote`.
  - current checkpoints:
    - `cargo fmt --check`: pass
    - `cargo clippy --all-targets -- -D warnings`: pass (0 error)
    - `cargo test --all-targets`: pass

## Exit Criteria
- File hotspot utama turun kompleksitasnya.
- Behavior tetap konsisten (test hijau).
