# Release Troubleshooting Guide

Complete troubleshooting guide for common release issues.

---

## 🔧 ISSUE 1: Cargo.lock Version Mismatch

### Symptoms:
```
error: the lock file needs to be updated but --locked was passed
```

### Cause:
- Version bumped in `Cargo.toml` but `Cargo.lock` not updated
- PKGBUILD uses `--locked` flag which requires lock file to match

### Solution:
```bash
# In main repo
cargo update
git add Cargo.lock
git commit -m "chore: update Cargo.lock for v0.X.X"
git push origin main

# Re-create tag (if already pushed)
git tag -d v0.X.X
git push origin :refs/tags/v0.X.X
git tag -a v0.X.X -m "Release v0.X.X"
git push origin v0.X.X
```

### Prevention:
- Always run `cargo update` after version bump
- Commit `Cargo.lock` together with `Cargo.toml` version change

---

## 🔧 ISSUE 2: SHA256 Checksum Mismatch

### Symptoms:
```
==> Validating source files with sha256sums...
    declarch-0.X.X.tar.gz ... FAILED
```

### Cause:
- Tag created before all commits pushed
- Tarball cached on GitHub doesn't match local files

### Solution:
```bash
# Wait 1-2 minutes for GitHub to update tarball
sleep 120

# Get correct SHA256 from actual tarball
curl -sL "https://github.com/nixval/declarch/archive/refs/tags/v0.X.X.tar.gz" | sha256sum

# Update PKGBUILD
vim PKGBUILD  # Paste correct SHA256

# Re-generate .SRCINFO
makepkg --printsrcinfo > .SRCINFO
```

### Alternative: Use makepkg to generate
```bash
makepkg -g 2>&1 | grep "^[a-f0-9]{64}"
```

---

## 🔧 ISSUE 3: AUR Repo Directory Contamination

### Symptoms:
- AUR repo contains source files (Cargo.toml, src/, etc.)
- `git status` shows hundreds of untracked files
- Build fails in weird ways

### Cause:
- Extracted tarball in AUR repo directory
- Run `makepkg` in wrong directory

### Solution:
```bash
# Clean AUR repo completely
cd ~/aur/declarch  # or your AUR repo path
git clean -fdx
git reset --hard HEAD

# Verify only PKGBUILD and .SRCINFO exist
ls -la
# Should show: .git, .SRCINFO, PKGBUILD

# Copy fresh PKGBUILD from main repo
cp /path/to/main/declarch/PKGBUILD .

# Generate .SRCINFO
makepkg --printsrcinfo > .SRCINFO

# Build from clean state
makepkg -si
```

### Prevention:
- NEVER extract tarball in AUR repo
- ALWAYS keep AUR repo minimal (only PKGBUILD + .SRCINFO)
- Use separate work directory for testing

---

## 🔧 ISSUE 4: Clippy Linting Failures in CI

### Symptoms:
- GitHub Actions fails at "Build & Test: Clippy"
- Error: `clippy::type_complexity` or similar

### Solution:
```bash
# Run clippy locally first
cargo clippy -- -D warnings

# Fix all warnings
# Example: type_complexity
# Add type aliases in code:
type MyComplexType = HashMap<Key, Value>;

# Re-test
cargo clippy -- -D warnings
cargo test

# Commit fix
git add .
git commit -m "fix: resolve clippy warnings"
git push origin main

# Re-create tag if needed
git tag -d v0.X.X
git push origin :refs/tags/v0.X.X
git tag -a v0.X.X -m "Release v0.X.X"
git push origin v0.X.X
```

### Prevention:
- Run `cargo clippy -- -D warnings` before every release
- Add clippy check to pre-commit hook (optional)

---

## 🔧 ISSUE 5: Slow Compilation (10+ minutes)

### Symptoms:
- `makepkg` takes forever
- Development builds too slow

### Solution:
```bash
# Use dev-release profile for testing
cargo build --profile dev-release  # 5-10 seconds

# Use release profile only for final binary
cargo build --release  # 10+ minutes, optimized
```

### PKGBUILD already uses release profile with optimizations:
- `lto = "thin"` - Link-time optimization
- `codegen-units = 1` - Max optimization
- `opt-level = "s"` - Optimize for size

### Don't change PKGBUILD build profile!

---

## 📋 COMPLETE RELEASE CHECKLIST

Use this checklist for every release:

### Pre-Release:
- [ ] Run `cargo clippy -- -D warnings` (fix all warnings)
- [ ] Run `cargo test` (all tests pass)
- [ ] Update version in `Cargo.toml`
- [ ] Run `cargo update` (update Cargo.lock)
- [ ] Update version in `PKGBUILD` (pkgver=0.X.X, pkgrel=1)
- [ ] Update version in `README.md` (badge)
- [ ] Commit all changes: `git commit -m "chore: bump version to 0.X.X"`

### Tag & Push:
- [ ] Push to GitHub: `git push origin main`
- [ ] Create tag: `git tag -a v0.X.X -m "Release v0.X.X"`
- [ ] Push tag: `git push origin v0.X.X`
- [ ] Wait 1-2 minutes for GitHub tarball to update

### Verify Release:
- [ ] Download and check tarball: `curl -L "https://github.com/nixval/declarch/archive/refs/tags/v0.X.X.tar.gz" | sha256sum`
- [ ] Update PKGBUILD sha256sums with correct hash
- [ ] Commit PKGBUILD: `git commit -m "fix: update SHA256 for v0.X.X"`
- [ ] Push again: `git push origin main`

### AUR Publish:
- [ ] Go to AUR repo: `cd ~/aur/declarch`
- [ ] Clean if needed: `git clean -fdx && git reset --hard HEAD`
- [ ] Copy PKGBUILD: `cp /path/to/main/declarch/PKGBUILD .`
- [ ] Generate .SRCINFO: `makepkg --printsrcinfo > .SRCINFO`
- [ ] Test build: `makepkg -si` (THIS TAKES 10+ MINUTES)
- [ ] Test binary: `declarch --version`
- [ ] Commit: `git add PKGBUILD .SRCINFO`
- [ ] Commit message: `git commit -m "upgpkg: 0.X.X"`
- [ ] Push: `git push`

### Post-Release:
- [ ] Create GitHub Release (web or `gh release create v0.X.X`)
- [ ] Update CHANGELOG.md
- [ ] GitHub Pages should auto-deploy

---

## 🚨 EMERGENCY: Rollback Release

If critical bug found after release:

### Option 1: Yank Release (Keep Tag)
- Go to GitHub Releases → Edit → "Yank release"
- Tag remains, but marked as pre-release/invalid

### Option 2: Delete Release and Tag
```bash
gh release delete v0.X.X
git push origin :refs/tags/v0.X.X
git tag -d v0.X.X
```

### Option 3: Hotfix with pkgrel (Fastest)
```bash
# In AUR repo
vim PKGBUILD  # Change pkgrel=1 to pkgrel=2
makepkg --printsrcinfo > .SRCINFO
makepkg -si
git add PKGBUILD .SRCINFO
git commit -m "fix: hotfix critical bug"
git push
```

### Option 4: Patch Release (v0.X.1)
```bash
# Fix bug in main repo
git add .
git commit -m "fix: critical bug"

# Bump version to 0.X.1
# Follow normal release process
```

---

## 📁 Directory Structure Reference

```
~/
├── aur/                          # AUR repositories (cloned from aur.archlinux.org)
│   ├── declarch/                 # Main AUR package
│   │   ├── .git/                 # Git metadata
│   │   ├── PKGBUILD              # ONLY THIS FILE
│   │   └── .SRCINFO              # AND THIS FILE
│   ├── declarch-bin/             # Binary AUR package
│   └── declarch-git/             # Git AUR package
│
└── github/
    └── repo/nixval/tools/
        └── declarch/             # Main repository
            ├── .aur/             # Gitignored! Personal reference only
            │   ├── templates/    # PKGBUILD templates for reference
            │   └── work/         # Temporary build files (can delete)
            ├── Cargo.toml        # Source code
            ├── PKGBUILD          # Master PKGBUILD (copy this to AUR)
            └── ...
```

**Important Rules:**
1. AUR repos (`~/aur/declarch`) should ONLY contain PKGBUILD + .SRCINFO
2. NEVER extract tarball in AUR repo
3. Use `.aur/work/` for testing builds, NOT in AUR repo
4. Always copy PKGBUILD from main repo to AUR repo

---

## 🔗 Useful Commands

```bash
# Check current version in all files
grep -r "0\.[0-9]\+\.[0-9]\+" Cargo.toml PKGBUILD README.md

# Generate SHA256 for current PKGBUILD
makepkg -g 2>&1 | grep -E "^[a-f0-9]{64}"

# Check if Cargo.lock needs update
cargo check --locked 2>&1 | grep -i "needs to be updated"

# Clean all build artifacts
cargo clean
makepkg -C

# Verify tarball before using
curl -L "https://github.com/.../v0.X.X.tar.gz" | tar tzv | head -20

# Test binary after install
declarch --version
declarch help
declarch check --config examples/minimal.kdl
```

---

**Last Updated:** 2026-01-23
**Maintainer:** nixval
