# Internal Module Boundaries and Extension Points

This document is a maintainer-oriented map of the post-refactor code boundaries.
Goal: keep command entrypoints orchestration-only and move operational logic into focused modules.

## 1) Config Loading (`src/config/loader.rs`)

Orchestration entrypoint:
- `load_root_config_with_selectors`
- `recursive_load`

Internal layers:
- `src/config/loader/import_context.rs`
  - Circular import detection and visited-set behavior.
- `src/config/loader/path_resolution.rs`
  - Path expansion, canonicalization, import path safety checks.
- `src/config/loader/merging.rs`
  - Raw-to-merged config accumulation and import queue extraction.
- `src/config/loader/selector_filter.rs`
  - Profile/host selector block expansion.

Invariant:
- Merge behavior is additive/order-preserving compatible with prior loader behavior.

## 2) CLI Dispatch (`src/cli/dispatcher.rs`)

Orchestration entrypoint:
- `dispatch`

Internal layers:
- `src/cli/dispatcher/output_contract.rs`
  - Structured output contract/version gates.
- `src/cli/dispatcher/normalization.rs`
  - Flag normalization and argument conversion helpers.
- `src/cli/dispatcher/routing.rs`
  - Command-family routing handlers.

Invariant:
- Machine-output policy is centralized before command execution.

## 3) Generic Backend Runtime (`src/backends/generic.rs`)

Orchestration entrypoint:
- `impl PackageManager for GenericManager`

Internal layers:
- `src/backends/generic/runtime.rs`
  - Binary resolution, command templating, process execution normalization.
- `src/backends/generic/command_exec.rs`
  - Timeout-aware process adapters (interactive and non-interactive).
- `src/backends/generic/search_parsing.rs`
  - Search output parsing pipeline by backend format.

Invariant:
- Parsing path and process execution path remain decoupled.

## 4) KDL Parser Layering (`src/config/kdl_modules/parser.rs`)

Orchestration entrypoint:
- `parse_kdl_content_with_path`

Internal layers:
- `src/config/kdl_modules/parser/ast_scan.rs`
  - KDL AST parse + enriched error reporting hook.
- `src/config/kdl_modules/parser/semantic_mapping.rs`
  - Node-to-config semantic mapping and normalization rules.

Invariant:
- KDL error diagnostics keep the same enriched formatting path.

## 5) Update Check (`src/utils/update_check.rs`)

Orchestration entrypoint:
- `update_hint_cached`
- `latest_version_live`

Internal layers:
- `src/utils/update_check/fetcher.rs`
  - Release source fetch + payload parsing.
- `src/utils/update_check/cache_policy.rs`
  - Cache TTL/fallback policy.
- `src/utils/update_check/owner_detection.rs`
  - Install owner/channel strategy.
- `src/utils/update_check/hint.rs`
  - Hint model generation.
- `src/utils/update_check/versioning.rs`
  - Version parsing and comparison.

Invariant:
- Network failure/offline path falls back to cache deterministically.

## Extension Points

### A) Add a new backend integration

Primary files:
- `src/backends/config.rs`
- `src/backends/registry.rs`
- `src/backends/generic.rs` + `src/backends/generic/runtime.rs`

Checklist:
1. Register backend config shape and command templates.
2. Ensure `GenericManager` command placeholders (`{binary}`, `{packages}`, `{query}`, `{repos}`) are sufficient.
3. Add/adjust parser config if custom output format is needed.
4. Add unit tests for selection/support capability and runtime behavior.

### B) Extend machine output contracts

Primary files:
- `src/cli/dispatcher/output_contract.rs`
- `src/utils/machine_output.rs`
- `docs/contracts/v1/`

Checklist:
1. Gate new command support in dispatcher contract checks.
2. Emit payload through `machine_output::emit_v1`.
3. Add or update contract example JSON/YAML docs under `docs/contracts/v1`.

### C) State IO invariants

Primary files:
- `src/state/io.rs`
- `src/state/io/load_recovery.rs`
- `src/state/io/migration.rs`
- `src/state/io/persist.rs`

Invariants to preserve:
1. Canonical state key format remains `backend:name`.
2. Strict vs non-strict load behavior must remain explicit.
3. Migration/sanitization must be idempotent.
4. Locking behavior must guard mutating writes.
