# AUR Release Automation

## Directory Structure

```
.aur/
├── templates/          # PKGBUILD templates (committed to git)
│   ├── PKGBUILD        # Template for declarch
│   ├── PKGBUILD-bin    # Template for declarch-bin
│   └── PKGBUILD-git    # Template for declarch-git
├── scripts/            # Automation scripts (gitignored)
│   ├── release.sh      # Complete release automation
│   └── publish.sh      # AUR publish automation
├── work/               # Working directory (gitignored)
│   ├── declarch/
│   ├── declarch-bin/
│   └── declarch-git/
└── README.md           # This file
```

## Release Workflow

### Method 1: Full Automation (Recommended)

Complete release process with one command:

```bash
# From repository root
./.aur/scripts/release.sh 0.4.0
```

This script will:
1. ✓ Run tests
2. ✓ Build release binary
3. ✓ Prepare all PKGBUILDs
4. ✓ Generate sha256sums
5. ✓ Test build all PKGBUILDs
6. ✓ Commit and push to main
7. ✓ Create and push tag to GitHub
8. ✓ Trigger GitHub Actions
9. ✓ Wait for you to confirm GitHub Actions completion

### Method 2: Step by Step

If you prefer manual control:

#### Step 1: Prepare Release
```bash
./.aur/scripts/release.sh 0.4.0
```

#### Step 2: Wait for GitHub Actions
Monitor at: https://github.com/nixval/declarch/actions

Wait until all checks pass (±10 minutes).

#### Step 3: Publish to AUR
```bash
./.aur/scripts/publish.sh 0.4.0
```

This script will:
1. ✓ Verify GitHub release exists
2. ✓ Download binary release
3. ✓ Update declarch-bin PKGBUILD sha256
4. ✓ Test build declarch-bin
5. ✓ Publish all 3 packages to AUR:
   - declarch
   - declarch-bin
   - declarch-git

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

**Problem**: Tag v0.4.0 already exists

**Solution**: The script will ask if you want to delete and recreate.
- Type `y` to delete and recreate (if you made changes)
- Type `N` to abort (if tag is correct)

## Quick Reference

| Command | Purpose |
|---------|---------|
| `./.aur/scripts/release.sh 0.4.0` | Complete release automation |
| `./.aur/scripts/publish.sh 0.4.0` | Publish to AUR (after GitHub Actions) |
| `https://github.com/nixval/declarch/actions` | Monitor GitHub Actions |

## Version Auto-Detection

Scripts can automatically detect version from `Cargo.toml`:

```bash
./.aur/scripts/release.sh      # Uses version from Cargo.toml
./.aur/scripts/publish.sh      # Uses version from Cargo.toml
```

Or specify version explicitly:

```bash
./.aur/scripts/release.sh 0.4.0
./.aur/scripts/publish.sh 0.4.0
```
