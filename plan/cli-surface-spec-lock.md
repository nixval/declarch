# CLI Surface Spec Lock (Phase A)

- Date: 2026-02-16
- Source of truth: `plan/rfc-flag-subcommand-surface-simplification.md`
- Goal: lock canonical command/flag patterns before implementation

## Chapter 1: Command Surface Audit

| Command Area | Current Shape | Issue | Decision |
|---|---|---|---|
| `sync` | `sync`, `sync sync`, `sync preview`, `sync update`, `sync prune`, `sync cache`, `sync upgrade` | `sync sync` redundant, `sync preview` overlaps with global dry-run | Remove `sync sync` and `sync preview`; keep `sync`, `update`, `prune`, `cache`, `upgrade` |
| `switch` | local `--dry-run` | Inconsistent with global dry-run contract | Remove local flag, use global `--dry-run` |
| `edit` | `--preview` + global `--dry-run` | Similar terms but different intent | Keep both, but define strict semantics |
| `sync` | `--gc` on parent + subcommands | Duplicate flag placement | Keep one canonical placement (subcommand-level for mutating sync flows) |
| `info` | mode multiplexing via many flags | Discoverability density | Keep for now, but tighten help and examples for beginner path |

## Chapter 2: Canonical Invocation Set

### 2.1 Keep
- `declarch sync`
- `declarch --dry-run sync`
- `declarch sync update`
- `declarch --dry-run sync update`
- `declarch sync prune`
- `declarch --dry-run sync prune`
- `declarch sync cache`
- `declarch sync upgrade`
- `declarch edit --preview`
- `declarch --dry-run edit ...`

### 2.2 Remove
- `declarch sync sync`
- `declarch sync preview`
- `declarch switch --dry-run ...`

## Chapter 3: Flag Semantics Contract

### 3.1 Global `--dry-run`
- Only canonical no-apply signal.
- Must affect all mutating flows:
  - `sync` family (except pure read-only subcommands),
  - `install`,
  - `switch`,
  - edit operations that may mutate files.

### 3.2 `edit --preview`
- Output mode only.
- Renders content (with optional line numbers), does not imply mutation flow.
- Can coexist with `--dry-run`, but help text should clarify primary intent.

### 3.3 `--verbose`
- Must have command-specific guaranteed extra diagnostics.
- If no meaningful diagnostics exist, command should not claim verbose value.

## Chapter 4: Output Contract (Beginner vs Diagnostic)

| Command | Non-Verbose (default) | Verbose (`--verbose`) |
|---|---|---|
| `sync` | concise plan/apply status + fix hint on failure | include target resolution, module/profile selectors, backend decisions, failure chain, timing |
| `install` | package add summary + sync outcome + fix hints | include module selection logic, backend resolution detail, rollback reason detail |
| `switch` | transition summary + clear failure remediation | include backend detection detail, dependency check trace, state lock/state save detail |
| `info` | concise status and key counts | include path locations, filters applied, data source trace |
| `info --plan` / query | concise reason output | include query resolution path and source file details |
| `search` | result-first display, minimal noise | include per-backend timing and skip reasons |
| `lint` | issue summary + next action | include mode/filter/runtime details and deeper context |
| `cache` / `upgrade` | action summary + explicit failed backend names | include backend capability checks and detailed command failure reason |

## Chapter 5: Additional Findings to Adjust

1. `info --doctor` currently prints runtime paths by default.
- Proposed: move path-heavy sections behind `--verbose`, keep default doctor focused on health checks and fix steps.

2. `install --dry-run` exits too early with raw package echo.
- Proposed: dry-run should still run normalization/resolution steps and print effective plan (backend/module), without writing files.

3. `info` and `info_reason` verbose depth is currently weak.
- Proposed: add meaningful diagnostic payload (selector context, matched source paths, fallback decisions).

4. `sync --gc` duplication (parent + subcommand) should be collapsed.
- Proposed: one placement only to reduce ambiguity and parser clutter.

## Chapter 6: Implementation Phases

## Phase 1: Parser cleanup
- Remove redundant subcommands/flags from `src/cli/args.rs`.
- Remove deprecated conversion/warning bridge from `src/cli/deprecated.rs` if no longer needed.

## Phase 2: Dispatcher alignment
- Enforce global `--dry-run` for `switch` and all mutating paths.
- Remove branching paths for removed forms.

## Phase 3: Output-tier normalization
- Trim default noise.
- Move diagnostics to verbose mode.
- Add remediation hints to key error paths.

## Phase 4: Docs/help cleanup
- Rewrite examples to canonical forms only.
- Keep quick-start minimal.

## Phase 5: Test hardening
- Negative parser tests for removed forms.
- Contract tests for default vs verbose output delta.

## Chapter 7: Done Criteria

- Removed forms fail fast with clear error message.
- All canonical forms documented and tested.
- Non-verbose output no longer dumps unnecessary internals.
- Verbose output consistently exposes diagnostics across commands.
