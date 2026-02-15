# RFC 0002: Cross-Platform Compatibility (macOS + Windows)

Status: Draft  
Author: declarch maintainers  
Created: 2026-02-15

## Summary

This RFC defines a low-risk path to support macOS and Windows over time, while keeping current Linux behavior stable.

The strategy is to isolate platform-specific behavior behind small utility abstractions first, then gradually add backend coverage and UX improvements.

## Goals

- Keep existing Linux workflow unchanged.
- Avoid large one-shot refactors.
- Make future macOS/Windows work incremental and testable.
- Keep `declarch` explicit and user-driven (no hidden inference).

## Non-goals

- Full parity for all Linux backends on day one.
- Replacing native package manager behavior.
- Adding mandatory daemon/runtime dependencies.

## Current baseline

- Paths already use `directories` crate (good base).
- Command execution still has Unix assumptions in several areas (`sudo`, shell wrappers).
- Backend ecosystem is Linux-heavy.

## Proposed architecture path

### 1. Platform command abstraction (Phase A)

Introduce a small `utils::platform` layer for:

- shell execution wrappers
- elevated command wrappers

This phase is intentionally minimal and should preserve existing Linux behavior exactly.

### 2. Capability flags per backend (Phase B)

Add explicit backend capability metadata (example):

- supported_os = ["linux", "macos", "windows"]
- requires_admin = true/false

Runtime should fail early with clear messages if user selects backend unsupported on host OS.

### 3. Native elevation model (Phase C)

- Linux/macOS: keep sudo-based flow where applicable.
- Windows: introduce a dedicated elevation path (PowerShell RunAs/UAC strategy) as an explicit backend/runtime feature, not hidden magic.

### 4. Backend onboarding by ecosystem (Phase D)

- macOS first: brew + cask-focused path.
- Windows first wave: winget/choco/scoop as independent backends.
- Keep backend files versioned and user-overridable, same as Linux model.

### 5. UX/documentation hardening (Phase E)

- Add per-OS quick start docs.
- Add doctor/lint hints for unsupported backend usage.
- Keep beginner docs straightforward; push complexity to advanced docs.

## Safety and compatibility rules

- New platform behavior must be opt-in unless it is a transparent bug fix.
- Do not auto-convert commands between shells silently.
- Prefer explicit error over implicit fallback if behavior is ambiguous.

## Testing strategy

- Keep Linux CI as primary gate initially.
- Add compile/test matrix for macOS and Windows early.
- Add targeted unit tests for platform abstraction layer.

## Short-term action items

1. Land platform command utility and migrate major `sudo` call-sites.
2. Add backend OS capability field and validation checks.
3. Add docs section: "Future Windows/macOS support path".

## Open questions

1. Should Windows elevation be implemented at core runtime or per-backend command templates?
2. Should backend capability be strict error by default or warning + skip?
3. Do we need shell-type field in backend config for cmd/powershell/bash differences?
