# Release Checklist

Use this checklist before tagging a release.

## 1) Quality Gates (mandatory)

Run all commands locally and ensure success:

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
```

## 2) Changelog Discipline

- Update `CHANGELOG.md` under `## [Unreleased]`.
- Ensure user-facing behavior changes are documented.
- Ensure breaking/risky changes include migration notes.

## 3) Phase Completion Gate

Confirm roadmap phase docs are complete:

- `plan/phase-01-security-correctness.md`
- `plan/phase-02-maintainability-modularity.md`
- `plan/phase-03-performance-reliability.md`
- `plan/phase-04-beginner-experience.md`
- `plan/phase-05-release-guardrails.md`

## 4) Rollback Readiness

- Review `plan/rollback-map.md`.
- Ensure each release-relevant batch commit can be reverted independently.
- Prefer `git revert <commit>` for safe rollback (avoid history rewrite on shared branches).

## 5) Freeze Window (regression watch)

Before final public announcement, keep a short freeze window (recommended: 24 hours):

- Accept only blocker fixes.
- Re-run quality gates after each blocker fix.
- Re-check smoke commands:

```bash
cargo run -- --help
cargo run -- sync --help
cargo run -- info --doctor
```

## 6) Tag + Publish

- Use `scripts/release.sh X.Y.Z` after the checklist is complete.
- Verify release assets for Linux/macOS/Windows in GitHub Releases.
