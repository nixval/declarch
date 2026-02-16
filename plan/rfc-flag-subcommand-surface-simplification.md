# RFC: Flag & Subcommand Surface Simplification

- Status: Draft (Hard-Cut Policy)
- Author: Codex
- Date: 2026-02-16
- Scope: CLI UX and command-surface consistency (`src/cli/*`, selected command contracts)
- Non-goal: no behavior-changing implementation in this RFC stage

## 1. Background

CLI surface has grown and now contains overlapping entrypoints for similar behavior. This increases onboarding cost, causes expectation mismatch, and expands test/maintenance matrix.

This RFC locks a minimal and consistent command pattern, with beginner-first default output and explicit diagnostic depth only on `--verbose`.

## 2. Key Problems (Observed)

1. No-op semantics are split between multiple shapes:
- `sync preview` vs global `--dry-run`.
- `switch --dry-run` (local) vs global `--dry-run`.

2. Redundant subcommand shape exists:
- `sync sync` duplicates parent default `sync` behavior.

3. `--verbose` value is inconsistent:
- Some commands provide meaningful extra data.
- Others only print minimal extra signal (or almost none), so users perceive no difference.

4. Default output still exposes details that many beginners do not need.

## 3. Design Goals

1. One concept, one canonical entrypoint.
2. Hard-cut redundant forms when identified as unnecessary.
3. Beginner-first default output: short, actionable, remediation-focused.
4. Diagnostic depth only behind `--verbose`.
5. Reduce parser/help/test surface area.

## 4. Non-Goals

1. No backend orchestration rewrite.
2. No expansion of machine contract scope during this simplification pass.

## 5. Canonical Contracts

### 5.1 No-apply contract

- Canonical no-op behavior is global `--dry-run`.
- Any mutating command must honor global `--dry-run` consistently.
- Command-local `--dry-run` variants should be removed.

### 5.2 Subcommand minimalism contract

- Keep only subcommands with distinct behavior intent.
- Remove alias-like subcommands that duplicate parent/default flow.

### 5.3 Output-tier contract

- Non-verbose output must contain:
  - status (success/fail),
  - short reason,
  - immediate fix step when failed.
- Non-verbose output should avoid unnecessary internals:
  - full path dumps,
  - backend trace internals,
  - source/fetch trace chains.
- `--verbose` must include diagnostics:
  - path/location details,
  - backend/source resolution details,
  - deeper failure chain,
  - optional timing data.

## 6. Hard Decisions (Final)

1. `declarch sync preview`
- Remove from canonical surface.
- Canonical path: `declarch --dry-run sync`.

2. `declarch sync sync`
- Remove (redundant with parent default `declarch sync`).

3. `declarch switch --dry-run` (local)
- Remove local flag.
- Use global `declarch --dry-run switch ...`.

4. `declarch edit --preview`
- Keep as first-class, because it is output mode (file rendering), not only execution guard.

5. Deprecation warnings
- Do not add deprecation warning layer for removed redundant shapes.
- Prefer direct hard-cut for consistency and reduced UX noise.

## 7. Additional Redundancy / Inconsistency Findings

1. `info` modes are multiplexed by flags (`--doctor`, `--plan`, `--list`) and still need clear mode contracts.
- `--scope` has replaced old `--orphans/--synced` flags and reduced ambiguity.

2. `install --dry-run` required richer planning output.
- This is now aligned to show resolved plan details (backend/module intent) without writing files.

3. Verbose quality varies by command:
- `search`, `cache`, `upgrade`, `lint` have clearer verbose deltas.
- `info` and `info_reason` now expose baseline context in verbose mode, but cross-command depth still needs standardized contract tests.

4. `sync --gc` was removed.
- Reason: runtime no-op and overlapped conceptually with `sync cache`.

## 8. Migration Plan (Phase-by-Phase)

## Phase A: Spec Lock

Deliverables:
- Command-by-command matrix:
  - canonical invocation,
  - removed forms,
  - allowed flags,
  - default output contract,
  - verbose output contract.

Exit criteria:
- Maintainer-approved matrix.

## Phase B: Parser Surface Simplification

Deliverables:
- Remove redundant parser branches/subcommands/flags directly.
- Keep only canonical forms.

Exit criteria:
- CLI parser no longer accepts removed forms.

## Phase C: Dispatcher & Behavior Alignment

Deliverables:
- Route all mutating commands through global `--dry-run` behavior.
- Standardize output tiers (default concise, verbose diagnostic).

Exit criteria:
- Consistent dry-run and verbose behavior across target commands.

## Phase D: Help & Docs Normalization

Deliverables:
- Help examples use only canonical forms.
- Remove old patterns from README/docs.

Exit criteria:
- Help/docs fully match implemented parser surface.

## Phase E: Test Matrix Hardening

Deliverables:
- Parser tests for allowed/removed forms.
- Behavior tests for default vs verbose contract.
- Snapshot tests for help text and high-signal errors.

Exit criteria:
- CI fails on contract drift.

## 9. Risk Assessment

1. Script breakage due to hard-cut
- Mitigation: publish clear migration note and examples in release notes.

2. Hidden behavior drift during simplification
- Mitigation: parity tests for canonical paths and golden output tests.

3. Beginner confusion from changing examples
- Mitigation: keep quick-start short and canonical only.

## 10. Acceptance Criteria

This RFC is accepted when:
1. Hard-cut removal set is approved.
2. Canonical command matrix is approved.
3. Output-tier contract is approved per command.
4. Test expectations for removed/kept forms are approved.

## 11. Implementation Note

No code behavior changes are made by this RFC document. Implementation follows in separate commits after approval.
