# AUR Release Automation

Detailed maintainer checklist:
- `./.aur/RELEASE_MAINTAINER.md`

## Directory Structure

```
.aur/
├── templates/          # PKGBUILD templates (committed to git)
│   ├── PKGBUILD        # Template for declarch
│   ├── PKGBUILD-bin    # Template for declarch-bin
│   └── PKGBUILD-git    # Template for declarch-git
├── scripts/            # AUR automation scripts
│   ├── publish-declarch.sh  # Publish source package to AUR
│   ├── publish.sh           # Legacy multi-package publish helper
│   └── release.sh           # Legacy release helper
├── work/               # Working directory (gitignored)
│   ├── declarch/
│   ├── declarch-bin/
│   └── declarch-git/
└── README.md           # This file
```

## Release Workflow

### Method 1: Direct Publish (Recommended for AUR source package)

Publish `declarch` source package with one command:

```bash
# From repository root
./.aur/scripts/publish-declarch.sh X.Y.Z
```

This script will:
1. ✓ Download GitHub release tarball for the requested version
2. ✓ Compute sha256
3. ✓ Regenerate PKGBUILD + .SRCINFO in `.aur/work/declarch`
4. ✓ Push update to `aur.archlinux.org/declarch`

### Method 2: Step by Step

If you prefer manual control:

#### Step 1: Prepare release artifacts
```bash
scripts/release.sh X.Y.Z
```

#### Step 2: Wait for GitHub Actions
Monitor at: https://github.com/nixval/declarch/actions

Wait until all checks pass (±10 minutes).

#### Step 3: Publish source package to AUR
```bash
./.aur/scripts/publish-declarch.sh X.Y.Z
```

This script will:
1. ✓ Check release consistency preflight
2. ✓ Download release source tarball
3. ✓ Update `PKGBUILD`/`.SRCINFO`
4. ✓ Publish `declarch` package to AUR

### Release consistency guard

Use this before release/publish:

```bash
scripts/check_release_consistency.sh --tag vX.Y.Z --strict
```

Optional remote info check:

```bash
scripts/check_release_consistency.sh --tag vX.Y.Z --check-aur-remote
```

## Templates

PKGBUILD files in `templates/` are templates used by automation scripts.

**DO NOT edit these directly** unless you want permanent changes.

The automation scripts will:
1. Copy templates to `work/` directory
2. Update version numbers
3. Generate sha256sums
4. Test build
5. Copy to root for commit

## Troubleshooting

### Script fails at "AUR publish failed"

**Problem**: SSH key requires passphrase

**Solution**: Load your SSH key into agent before running script:
```bash
ssh-add
# Enter your passphrase
```

Then run the script again.

### "GitHub release not found"

**Problem**: GitHub Actions hasn't finished yet

**Solution**: Wait for GitHub Actions to complete:
1. Go to: https://github.com/nixval/declarch/actions
2. Wait until all checks pass
3. Run publish script again

### "Tests failed"

**Problem**: Code changes broke tests

**Solution**: Fix failing tests, then run release script again.

### "Tag already exists"

**Problem**: Tag `vX.Y.Z` already exists

**Solution**: The script will ask if you want to delete and recreate.
- Type `y` to delete and recreate (if you made changes)
- Type `N` to abort (if tag is correct)

## Quick Reference

| Command | Purpose |
|---------|---------|
| `scripts/release.sh X.Y.Z` | Prepare release/tag from repo root |
| `./.aur/scripts/publish-declarch.sh X.Y.Z` | Publish `declarch` to AUR |
| `scripts/check_release_consistency.sh --tag vX.Y.Z --strict` | Validate release/AUR version consistency |
| `https://github.com/nixval/declarch/actions` | Monitor GitHub Actions |

## Version Auto-Detection

Scripts can automatically detect version from `Cargo.toml`:

```bash
scripts/release.sh                 # Uses version from Cargo.toml
./.aur/scripts/publish-declarch.sh # Uses version from Cargo.toml
```

Or specify version explicitly:

```bash
scripts/release.sh X.Y.Z
./.aur/scripts/publish-declarch.sh X.Y.Z
```
