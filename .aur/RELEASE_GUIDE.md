# Release Guide

This page is for preparing a new `declarch` release and AUR update.

## 1. Prepare Version

Run from repo root:

```bash
scripts/release.sh X.Y.Z
```

This updates release metadata, runs core checks, commits, and pushes tag `vX.Y.Z`.

## 2. Consistency Guard

Always verify release consistency before publishing:

```bash
scripts/check_release_consistency.sh --tag vX.Y.Z --strict
```

Optional remote check against AUR metadata:

```bash
scripts/check_release_consistency.sh --tag vX.Y.Z --check-aur-remote
```

The guard verifies:
- `Cargo.toml` version
- release tag version
- `.aur/templates/PKGBUILD` `pkgver`
- expected AUR source URL pattern

## 3. Wait For GitHub Release

Monitor:

- https://github.com/nixval/declarch/actions

Wait until release artifacts are published.

## 4. Publish AUR (declarch)

Then publish:

```bash
./.aur/scripts/publish-declarch.sh X.Y.Z
```

This regenerates `.SRCINFO` from PKGBUILD and pushes to `aur.archlinux.org/declarch`.

## 5. Post-release Checks

- Verify `declarch --version` from release artifact
- Verify AUR package page shows expected version
- Smoke check install script flow (`install.sh`, `install.ps1`)
