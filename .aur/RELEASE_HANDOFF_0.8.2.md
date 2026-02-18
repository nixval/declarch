# Release Handoff (0.8.2 line)

This file summarizes the implementation work completed on branch `feat/0.8.2-feature`.

## Scope Completed

- Hidden `self-update` command for script/manual installs
- Update notification in `info` flow with install-owner-aware hints
- Windows self-update bootstrap path (detached PowerShell installer runner)
- Central project identity abstraction (`src/project_identity.rs`) across runtime paths
- Legacy-aware env-key resolver support for identity transitions
- MCP tool-id stabilization using `STABLE_PROJECT_ID` + alias normalization
- Release/AUR consistency guard script and CI/workflow integration
- Public docs cleanup: keep release/process docs in `.aur/`, not `docs-book`
- Neutral wording cleanup in public docs (replacing role-heavy labels)

## Guardrails Added

- `scripts/check_identity_literals.sh`
- `scripts/check_release_consistency.sh`
  - Cargo version check
  - release tag check (`--tag`)
  - `.aur/templates/PKGBUILD` `pkgver` check
  - expected source URL pattern check
  - optional AUR remote info check

## Core Validation Performed

- `cargo fmt`
- `cargo test --all-targets --quiet`
- `./scripts/check_identity_literals.sh`
- `./scripts/check_release_consistency.sh --strict --tag v0.8.2`

## Quick Reviewer Checklist

1. `self-update` behavior:
   - package-manager-owned install should return manager-specific update hint
   - script/manual install should proceed through self-update flow
2. MCP behavior:
   - stable tool IDs still accepted
   - alias normalization path works when binary prefix differs from stable id
3. Release guardrails:
   - CI includes consistency guard
   - release workflow blocks tag/version mismatch
4. Docs scope:
   - no release-process guide in `docs-book`
   - release process docs present under `.aur/`

## Release Command Sequence

```bash
scripts/release.sh X.Y.Z
scripts/check_release_consistency.sh --tag vX.Y.Z --strict
./.aur/scripts/publish-declarch.sh X.Y.Z
```
