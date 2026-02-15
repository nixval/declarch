# RFC 0001: Simple but Powerful Roadmap

Status: Draft  
Author: declarch maintainers  
Created: 2026-02-15

## Summary

This RFC proposes four opt-in capabilities to make `declarch` more powerful while keeping core behavior simple and explicit:

1. `explain` command (decision transparency)
2. `lint` command (config quality checks)
3. `policy` extensions (explicit guardrails)
4. `lock` workflow (reproducibility)

It also defines an extension direction for plugin/API/MCP without forcing a large core refactor.

## Design principles

- Keep default behavior unchanged for existing users.
- No hidden auto-detection or "smart" inference.
- User intent must be explicit via config or CLI flags.
- New features should be opt-in and composable.
- Error messages should be actionable and beginner-readable.

## Non-goals

- Turning `declarch` into a full orchestration platform.
- Replacing native package manager semantics.
- Mandatory daemon/service architecture.
- Breaking current `sync/install/init/check/info/search` workflows.

## Proposal A: `declarch explain`

### Goal
Help users understand *why* a package is installed/pruned/conflicted.

### CLI draft

```bash
declarch explain <query>
declarch explain aur:bat
declarch explain --target sync-plan
```

### Output draft

- source files/modules contributing declaration
- selected backend and package identity
- active overlays (`--profile`, `--host`, `--modules`)
- duplicate/conflict notes
- install/prune decision reason

### Compatibility
Purely additive command.

## Proposal B: `declarch lint`

### Goal
Static quality checks on config style and safety.

### CLI draft

```bash
declarch lint
declarch lint --strict
declarch lint --fix
```

### Initial rules

- deprecated syntax usage (`packages` vs `pkg`)
- ambiguous declarations (missing backend where policy requires)
- duplicate declarations across modules
- hooks configured but experimental hook consent missing
- unresolved/unused imports

### Fix mode
`--fix` only applies safe rewrites (formatting/canonical ordering/syntax migrations), never runtime behavior changes.

## Proposal C: `policy` extensions

### Goal
Allow teams/users to enforce explicit guardrails.

### KDL draft

```kdl
policy {
  require-backend true
  forbid-hooks false
  allow-backend "aur" "flatpak" "npm"
  forbid-backend "pacman"
  on-duplicate "error" // warn | error
  on-conflict "error"  // warn | error
}
```

### Behavior

- defaults preserve current behavior
- policy applies deterministically
- violations fail fast with clear guidance

## Proposal D: `lock` workflow

### Goal
Provide reproducibility mode when users need deterministic sync.

### CLI draft

```bash
declarch lock
declarch lock --update
declarch sync --locked
```

### Lock file scope (v1)

- resolved package identities (`backend:name`)
- selected variants where applicable
- config hash/fingerprint
- optional backend metadata if available

### Behavior

- `sync --locked` refuses drift when lock is stale
- default `sync` remains unchanged

## Extension model (future, optional)

### 1. Plugin process model

- external binaries (e.g. `declarch-plugin-foo`)
- JSON stdin/stdout contract
- isolated failures, no core crash propagation

### 2. Local API mode

- optional `declarch daemon`
- local socket/http endpoints for UI/automation
- CLI remains primary interface

### 3. MCP server mode

- expose read-only and controlled write tools
- useful for editor/assistant integration
- maps well to `explain/lint/plan` operations

## Rollout plan

### Phase 1 (low risk)

- implement `explain` (read-only)
- implement `lint` (read-only + optional `--fix` safe only)

### Phase 2 (medium risk)

- implement policy keys incrementally
- enforce in `check`, `install`, and `sync`

### Phase 3 (higher risk)

- introduce lock schema + generation
- add `sync --locked`

### Phase 4 (optional ecosystem)

- plugin runtime contract
- API/MCP spike as separate experimental path

## Open questions

1. Should `policy` allow module-level overrides or root-only?
2. Should `lint --fix` auto-migrate `packages` legacy syntax by default?
3. For lockfile: single file at root vs per-module lock fragments?
4. Should plugin execution require explicit experimental flag like hooks?

## Decision request

Accept RFC as roadmap baseline and start Phase 1 on a new branch.
