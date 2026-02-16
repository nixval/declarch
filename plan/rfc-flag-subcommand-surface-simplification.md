# RFC: Flag & Subcommand Surface Simplification

- Status: Draft
- Author: Codex
- Date: 2026-02-16
- Scope: CLI UX and command-surface consistency (`src/cli/*`, selected command option contracts)
- Non-goal: no behavior-changing implementation in this RFC stage

## 1. Background

Current CLI has matured feature-wise, but command surface has grown with overlapping semantics across global flags, command-local flags, and subcommands. This creates a higher cognitive load, especially for new users, and increases maintenance/testing burden.

This RFC documents the observed problems and defines a phased simplification plan before implementation.

## 2. Problem Statement

### 2.1 Redundant or overlapping semantics

1. `sync preview` vs global `--dry-run`
- Both communicate no-op planning behavior.
- Today they are related but not perceived as a single canonical workflow.

2. `edit --preview` vs global `--dry-run`
- `edit --preview` is preview-oriented, while global `--dry-run` also signals no-apply behavior.
- Dual vocabulary (`preview` and `dry-run`) risks confusion.

3. `switch --dry-run` local flag vs global `--dry-run`
- `switch` defines a local `--dry-run`, while other commands rely on the global flag.
- Inconsistent placement weakens predictability.

### 2.2 Discoverability and expectation mismatch

1. `--verbose` appears uneven
- Several commands accept and propagate verbose option, but user-perceived differences are inconsistent.
- Without a minimum verbose contract, users cannot reliably expect “what extra appears”.

2. `sync` surface area remains wide
- `sync` parent mode + multiple subcommands + deprecated bridge logic adds conceptual overhead.
- This is manageable for advanced users but not beginner-friendly by default.

### 2.3 Maintenance and test complexity

- Larger command surface multiplies matrix size for help text, parser validity, behavior compatibility, and machine-output contracts.
- Overlap increases risk of regressions where one path gets fixed while another equivalent path diverges.

## 3. Design Goals

1. One concept, one canonical entrypoint whenever possible.
2. Keep migration low-risk by using deprecation windows instead of sudden removal.
3. Preserve advanced capabilities while reducing beginner decision points.
4. Define explicit UX contracts for high-impact global flags (`--dry-run`, `--verbose`).
5. Keep machine-output support boundaries explicit and testable.

## 4. Non-Goals

1. No rewrite of backend orchestration.
2. No immediate removal of legacy pathways in this RFC stage.
3. No expansion of machine-output support in this effort unless needed by consistency contract.

## 5. Findings Matrix

| Area | Current State | Risk | Proposed Direction |
|---|---|---|---|
| Dry-run semantics | Mixed across global and local flags | Medium | Unify under global `--dry-run` contract |
| Preview semantics | Exists as subcommand and command-local flag | Medium | Keep `preview` only where output mode is materially different; otherwise alias/deprecate |
| Verbose behavior | Inconsistent perceived deltas | Medium | Define minimum verbose signal per command |
| Sync command surface | Rich but dense | Medium | Reduce overlap; document canonical paths |
| Help examples | Can drift from canonical UX | Low/Med | Align help with single recommended path per workflow |

## 6. Proposed Contract (Target State)

### 6.1 Canonical no-apply behavior

- Canonical keyword: `--dry-run` (global).
- Rule: any command that can mutate state MUST honor global `--dry-run` consistently.
- If command-local dry-run exists, convert to compatibility alias with deprecation warning.

### 6.2 Preview terminology policy

- `preview` keyword remains only when the command is fundamentally “render/report focused” and not equivalent to just no-apply execution.
- If behavior is equivalent to `--dry-run` execution path, avoid separate user-facing concept long-term.

### 6.3 Verbose minimum contract

Each command that accepts `--verbose` should expose at least one of:
- execution timing summary,
- decision trace (why action skipped/selected),
- backend resolution detail.

If no meaningful extra signal exists, command should not advertise verbose (or should be documented as no-op and then fixed).

## 7. Migration Plan (Phase-by-Phase)

## Phase A: Spec Lock

Deliverables:
- Write command-by-command contract table:
  - supports mutation?
  - supports `--dry-run`?
  - supports `preview`?
  - supports `--verbose` with guaranteed extra output?
- Mark target canonical usage per command.

Exit criteria:
- Maintainer-approved contract table.

## Phase B: Compatibility Layer

Deliverables:
- Keep old inputs operational but route to canonical path.
- Add precise deprecation warnings for redundant forms.
- Warnings must include replacement command example.

Exit criteria:
- Legacy syntax still works; warning appears once per invocation.

## Phase C: Help/Docs Normalization

Deliverables:
- `--help` examples show only canonical forms.
- README/docs quick-start references canonical path.
- Remove contradictory examples.

Exit criteria:
- No help/docs path promotes deprecated form as primary.

## Phase D: Test Matrix Hardening

Deliverables:
- Parser tests for canonical and compatibility forms.
- Behavioral parity tests for aliased/redundant inputs.
- Snapshot tests for help text and deprecation warnings.

Exit criteria:
- CI catches divergence between alias path and canonical path.

## Phase E: Removal Window

Deliverables:
- Remove deprecated forms after agreed grace period.
- Update changelog and migration note.

Exit criteria:
- Zero deprecated parser branches for removed items.

## 8. Candidate Redundancy Actions (Draft)

1. `switch --dry-run` (local)
- Action: keep temporarily as alias; internally map to global dry-run semantics; deprecate local form later.

2. `edit --preview` vs `--dry-run`
- Action: evaluate output distinction.
- If distinction is only naming, converge to one model; if distinction is meaningful (e.g., line-annotated patch preview), document strict role boundaries.

3. `sync preview` vs `--dry-run sync`
- Action: define canonical form for “plan report”.
- Keep alternative as compatibility alias during migration.

4. `--verbose`
- Action: audit each command for guaranteed extra output; add missing signal or remove claim from help.

## 9. Risk Assessment

1. Breaking scripts
- Mitigation: deprecation period + compatibility alias + explicit warnings.

2. User confusion during transition
- Mitigation: one-line replacement guidance on every warning.

3. Help/doc drift
- Mitigation: snapshot tests for key help surfaces.

4. Hidden behavior divergence
- Mitigation: parity tests between old/new invocation shapes.

## 10. Open Questions (for review)

1. Canonical sync preview path should be:
- A) `declarch sync preview` (report-first UX), or
- B) `declarch --dry-run sync` (single no-op concept globally)?

2. Should `edit --preview` stay as a first-class UX for editor-centric flow, with `--dry-run` treated as execution guard only?

3. Deprecation window target:
- one minor release or two minor releases?

## 11. Acceptance Criteria

This RFC is accepted when:
1. Canonical and compatibility invocation sets are approved.
2. Deprecation timeline is approved.
3. Test expectations for parser/help/parity are approved.

## 12. Implementation Note

No code behavior changes are made by this RFC document. Implementation will follow in separate, reviewable commits after approval.
