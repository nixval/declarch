# Declarch Release Guide

**Panduan lengkap release declarch ke GitHub dan AUR**

Last updated: 2025-01-14

---

## Table of Contents

1. [Quick Start](#quick-start)
2. [Prerequisites](#prerequisites)
3. [Complete Release Workflow](#complete-release-workflow)
4. [Troubleshooting](#troubleshooting)
5. [Reference](#reference)

---

## Quick Start

**Release ke GitHub + AUR dalam 3 langkah:**

```bash
# 1. Commit changes dan push ke main
git add .
git commit -m "Release v0.4.0"
git push origin main

# 2. Create dan push tag
git tag -a v0.4.0 -m "Release v0.4.0"
git push origin v0.4.0

# 3. Tunggu 10 menit, lalu publish ke AUR (lihat bagian AUR Publish di bawah)
```

---

## Prerequisites

### Required Tools

```bash
# Install tools
sudo pacman -S github cli
```

### SSH Key untuk AUR (WAJIB)

**Cek apakah SSH key sudah ada:**
```bash
ls -la ~/.ssh/*.pub
```

**Kalau belum ada, buat baru:**
```bash
# Generate SSH key (TANPA passphrase untuk kemudahan)
ssh-keygen -t ed25519 -C "nixval@aur" -f ~/.ssh/aur_key -N ""

# Add public key ke AUR account
cat ~/.ssh/aur_key.pub
# Copy output → https://aur.archlinux.org/account → SSH Public Keys → Paste

# Configure SSH
cat >> ~/.ssh/config << 'EOF'

Host aur.archlinux.org
    User aur
    IdentityFile ~/.ssh/aur_key
EOF

# Test connection
ssh aur@aur.archlinux.org
# Should show: "Welcome to AUR, nixval!"
```

**Kalau SSH key sudah ada tapi pakai passphrase:**
```bash
# Load key setiap kali mau publish
eval "$(ssh-agent -s)"
ssh-add ~/.ssh/id_nixval  # atau key lain yang ada
# ← Masukkan passphrase

# Test connection
ssh aur@aur.archlinux.org
```

---

## Complete Release Workflow

### STEP 1: Update Version Files

```bash
# Edit Cargo.toml
nano Cargo.toml
# Update: version = "0.4.0"

# Update PKGBUILD di root
nano PKGBUILD
# Update: pkgver=0.4.0

# Update PKGBUILD templates di .aur
nano .aur/templates/PKGBUILD
# Update: pkgver=0.4.0

nano .aur/templates/PKGBUILD-bin
# Update: pkgver=0.4.0

nano .aur/templates/PKGBUILD-git
# (pkgver function will auto-generate version)
```

### STEP 2: Run Tests

```bash
# Run semua tests
cargo test

# Expected: All tests pass
```

**JIKA TEST GAGAL:**
```bash
# Fix code
# ...

# Rerun tests
cargo test
```

### STEP 3: Build Binary

```bash
# Build release binary
cargo build --release

# Test binary
./target/release/declarch --version
./target/release/declarch check
```

### STEP 4: Prepare PKGBUILDs

```bash
# Create work directory
mkdir -p .aur/work/{declarch,declarch-bin,declarch-git}

# Copy dan update declarch PKGBUILD
sed "s/pkgver=.*/pkgver=0.4.0/" .aur/templates/PKGBUILD > .aur/work/declarch/PKGBUILD
cd .aur/work/declarch
makepkg -g >> PKGBUILD
cd -

# Copy dan update declarch-bin PKGBUILD
sed "s/pkgver=.*/pkgver=0.4.0/" .aur/templates/PKGBUILD-bin > .aur/work/declarch-bin/PKGBUILD

# Copy declarch-git PKGBUILD
cp .aur/templates/PKGBUILD-git .aur/work/declarch-git/PKGBUILD
```

### STEP 5: Test PKGBUILDs

```bash
# Test declarch PKGBUILD
cd .aur/work/declarch
makepkg -s
cd -

# Test declarch-git PKGBUILD
cd .aur/work/declarch-git
makepkg --nobuild
cd -
```

**JIKA BUILD GAGAL:**
```bash
# Cek error
cd .aur/work/declarch
makepkg -s

# Fix PKGBUILD
nano .aur/work/declarch/PKGBUILD

# Update template
cp .aur/work/declarch/PKGBUILD .aur/templates/PKGBUILD

# Test lagi
makepkg -s
```

### STEP 6: Commit Changes

```bash
# Cek status
git status

# Add semua changes
git add .

# Commit
git commit -m "Release v0.4.0

- Feature 1
- Feature 2
- Bug fixes"
```

### STEP 7: Push to Main

```bash
# Push ke main branch
git push origin main

# Verify
git status
# Should show: "Your branch is up to date with 'origin/main'"
```

### STEP 8: Create Tag

```bash
# Cek commit terbaru
git log -1 --oneline

# Create annotated tag
git tag -a v0.4.0 -m "Release v0.4.0

Major Features:
- Feature 1
- Feature 2
- Bug fixes"
```

### STEP 9: Push Tag

```bash
# Push tag ke GitHub
git push origin v0.4.0
```

**JIKA TAG SUDAH ADA:**
```bash
# Delete tag dulu
git tag -d v0.4.0
git push origin :refs/tags/v0.4.0

# Create tag lagi
git tag -a v0.4.0 -m "Release v0.4.0"
git push origin v0.4.0
```

### STEP 10: Wait for GitHub Actions

```bash
# Monitor GitHub Actions
gh run list --repo nixval/declarch --limit 5

# Atau visit: https://github.com/nixval/declarch/actions
```

**Wait until:**
- ✅ All workflows pass (green checkmark)
- ✅ Release created at: https://github.com/nixval/declarch/releases/tag/v0.4.0
- ✅ Binary artifacts uploaded (declarch-x86_64-unknown-linux-gnu.tar.gz)

**Wait time: ±10 minutes**

### STEP 11: Verify GitHub Release

```bash
# Cek release
gh release view v0.4.0

# Cek artifacts
gh release view v0.4.0 --json assets

# Download binary untuk cek SHA256
cd /tmp
wget https://github.com/nixval/declarch/releases/download/v0.4.0/declarch-x86_64-unknown-linux-gnu.tar.gz
sha256sum declarch-x86_64-unknown-linux-gnu.tar.gz
# Output: abc123...  declarch-x86_64-unknown-linux-gnu.tar.gz
```

### STEP 12: Update declarch-bin PKGBUILD SHA256

```bash
# Dapatkan SHA256 dari output langkah sebelumnya
# Contoh: abc123def456... adalah SHA256

cd .aur/work/declarch-bin

# Update PKGBUILD dengan SHA256 yang benar
sed -i "s/sha256sums=.*/sha256sums=('ABC123DEF456...')/" PKGBUILD

# Test build lagi
makepkg -s

cd -
```

### STEP 13: Generate .SRCINFO Files

```bash
# Generate .SRCINFO untuk semua packages
cd .aur/work/declarch
makepkg --printsrcinfo > .SRCINFO

cd ../declarch-bin
makepkg --printsrcinfo > .SRCINFO

cd ../declarch-git
makepkg --printsrcinfo > .SRCINFO

cd -
```

---

## AUR Publish

### STEP 14: Publish declarch ke AUR

```bash
# Load SSH key (kalau pakai passphrase)
eval "$(ssh-agent -s)"
ssh-add ~/.ssh/id_nixval
# ← Masukkan passphrase

# Clone atau buat AUR repo
cd /tmp
rm -rf aur-declarch
mkdir aur-declarch
cd aur-declarch
git init
git remote add origin aur@aur.archlinux.org:declarch.git
git pull origin master 2>/dev/null || echo "First time push"

# Copy files
cp /home/nixval/github/repo/nixval/tools/declarch/.aur/work/declarch/{PKGBUILD,.SRCINFO} .

# Add dan commit
git add PKGBUILD .SRCINFO
git commit -m "Update to 0.4.0"

# Push ke AUR
git push -u origin master
```

**Verify:**
```bash
# Cek di browser
# https://aur.archlinux.org/packages/declarch
# Harus muncul "Updated to 0.4.0"
```

### STEP 15: Publish declarch-bin ke AUR

```bash
cd /tmp
rm -rf aur-declarch-bin
mkdir aur-declarch-bin
cd aur-declarch-bin
git init
git remote add origin aur@aur.archlinux.org:declarch-bin.git
git pull origin master 2>/dev/null || echo "First time push"

# Copy files
cp /home/nixval/github/repo/nixval/tools/declarch/.aur/work/declarch-bin/{PKGBUILD,.SRCINFO} .

# Add dan commit
git add PKGBUILD .SRCINFO
git commit -m "Update to 0.4.0"

# Push ke AUR
git push -u origin master
```

**Verify:**
```bash
# https://aur.archlinux.org/packages/declarch-bin
```

### STEP 16: Publish declarch-git ke AUR

```bash
cd /tmp
rm -rf aur-declarch-git
mkdir aur-declarch-git
cd aur-declarch-git
git init
git remote add origin aur@aur.archlinux.org:declarch-git.git
git pull origin master 2>/dev/null || echo "First time push"

# Copy files
cp /home/nixval/github/repo/nixval/tools/declarch/.aur/work/declarch-git/{PKGBUILD,.SRCINFO} .

# Add dan commit
git add PKGBUILD .SRCINFO
git commit -m "Update to 0.4.0"

# Push ke AUR
git push -u origin master
```

**Verify:**
```bash
# https://aur.archlinux.org/packages/declarch-git
```

### STEP 17: Test Install dari AUR

```bash
# Test install declarch-bin (paling cepat)
paru -G declarch-bin
cd declarch-bin
makepkg -si

# Cek install
declarch --version
declarch check
```

---

## Troubleshooting

### Problem: Tests Failed

```
Error: Tests failed
```

**Solution:**
```bash
# Run tests dengan output
cargo test

# Fix failing tests
# ...

# Rerun
cargo test
```

---

### Problem: Tag Already Exists

```
Warning: Tag v0.4.0 already exists
```

**Solution:**
```bash
# Delete tag
git tag -d v0.4.0
git push origin :refs/tags/v0.4.0

# Create tag baru
git tag -a v0.4.0 -m "Release v0.4.0"
git push origin v0.4.0
```

---

### Problem: GitHub Actions Failed

```
Error: GitHub Actions build failed
```

**Solution:**
```bash
# Cek error
gh run view <run-id>
# Atau: https://github.com/nixval/declarch/actions

# Fix code
# ...

# Delete tag
git tag -d v0.4.0
git push origin :refs/tags/v0.4.0

# Delete release
gh release delete v0.4.0 -y

# Commit fix
git add .
git commit -m "Fix GitHub Actions build"
git push origin main

# Create tag baru
git tag -a v0.4.0 -m "Release v0.4.0"
git push origin v0.4.0
```

---

### Problem: SSH Permission Denied

```
Permission denied (publickey).
fatal: Could not read from remote repository.
```

**Solution:**
```bash
# Load SSH key (kalau pakai passphrase)
eval "$(ssh-agent -s)"
ssh-add ~/.ssh/id_nixval
# ← Masukkan passphrase

# Test connection
ssh aur@aur.archlinux.org
# Should show: "Welcome to AUR, nixval!"

# Retry push
git push
```

---

### Problem: PKGBUILD Build Failed

```
Error: PKGBUILD test failed
```

**Solution:**
```bash
# Cek error detail
cd .aur/work/declarch
makepkg -s

# Fix PKGBUILD
nano PKGBUILD

# Test lagi
makepkg -s

# Update template
cp PKGBUILD /home/nixval/github/repo/nixval/tools/declarch/.aur/templates/PKGBUILD
```

---

### Problem: SHA256 Mismatch

```
Error: sha256sums failed
```

**Solution:**
```bash
# Download binary lagi
cd /tmp
wget https://github.com/nixval/declarch/releases/download/v0.4.0/declarch-x86_64-unknown-linux-gnu.tar.gz

# Generate SHA256 yang benar
sha256sum declarch-x86_64-unknown-linux-gnu.tar.gz

# Update PKGBUILD
nano /home/nixval/github/repo/nixval/tools/declarch/.aur/work/declarch-bin/PKGBUILD
# Ganti sha256sums dengan output yang benar
```

---

### Problem: AUR Repository Not Found

```
fatal: repository 'https://aur.archlinux.org/declarch.git/' not found
```

**Solution:**

Ini berarti package belum ada di AUR. Kamu perlu upload pertama kali via web:

1. Visit: https://aur.archlinux.org/pkgbase/declarch
2. Click "Submit Package"
3. Upload PKGBUILD files via web interface
4. Setelah approved, baru bisa push via git

---

### Problem: .SRCINFO Generation Failed

```
Error: .SRCINFO generation failed
```

**Solution:**
```bash
# Cek PKGBUILD syntax
cd .aur/work/declarch
nano PKGBUILD

# Test generate
makepkg --printsrcinfo

# Kalau berhasil, save ke file
makepkg --printsrcinfo > .SRCINFO
```

---

### Problem: Non-Fast Forward Error

```
remote: error: denying non-fast-forward
```

**Solution:**
```bash
# Pull changes dulu
git pull origin master

# Atau force push (HATI-HATI)
git push -f origin master
```

---

## Error Recovery

### Scenario 1: Bug Found Setelah Release

```bash
# Fix bug
# ... edit code ...

# Commit fix
git add .
git commit -m "Fix critical bug"
git push origin main

# Create patch release (new version)
git tag -a v0.4.1 -m "Release v0.4.1 (bugfix)"
git push origin v0.4.1

# Ulangi release process untuk v0.4.1
```

---

### Scenario 2: Wrong Tag Version

```bash
# Delete tag
git tag -d v0.4.0
git push origin :refs/tags/v0.4.0

# Delete GitHub release
gh release delete v0.4.0 -y

# Create correct tag
git tag -a v0.4.1 -m "Release v0.4.1"
git push origin v0.4.1
```

---

### Scenario 3: PKGBUILD Error Setelah Release

```bash
# Fix PKGBUILD
nano .aur/work/declarch/PKGBUILD

# Generate .SRCINFO
cd .aur/work/declarch
makepkg --printsrcinfo > .SRCINFO

# Publish update ke AUR
cd /tmp/aur-declarch
cp /home/nixval/github/repo/nixval/tools/declarch/.aur/work/declarch/{PKGBUILD,.SRCINFO} .
git add PKGBUILD .SRCINFO
git commit -m "Fix PKGBUILD"
git push

# TIDAK perlu retag untuk PKGBUILD fixes
```

---

## Reference

### Useful Commands

```bash
# Check git tags
git tag -l

# View tag info
git show v0.4.0

# Delete local tag
git tag -d v0.4.0

# Delete remote tag
git push origin :refs/tags/v0.4.0

# Delete GitHub release
gh release delete v0.4.0 -y

# Check GitHub Actions
gh run list --repo nixval/declarch

# View GitHub Actions run
gh run view <run-id>

# Check AUR package
curl -s https://aur.archlinux.org/cgit/aur.git/plain/PKGBUILD?h=declarch

# Test PKGBUILD locally
makepkg -s

# Check package quality
namcap PKGBUILD *.pkg.tar.zst

# Generate .SRCINFO
makepkg --printsrcinfo > .SRCINFO
```

---

### Directory Structure

```
.aur/
├── templates/              # PKGBUILD templates (gitignored)
│   ├── PKGBUILD           # Source package
│   ├── PKGBUILD-bin       # Binary package
│   └── PKGBUILD-git       # Git package
└── work/                   # Working directory (gitignored)
    ├── declarch/
    │   ├── PKGBUILD       # Prepared for release
    │   └── .SRCINFO       # Generated
    ├── declarch-bin/
    │   ├── PKGBUILD
    │   └── .SRCINFO
    └── declarch-git/
        ├── PKGBUILD
        └── .SRCINFO
```

---

### AUR Package Types

- **declarch**: Builds from source tarball (stable)
- **declarch-bin**: Pre-built binary from GitHub (quick install)
- **declarch-git**: Builds from latest git commit (bleeding edge)

---

### Pre-Release Checklist

- [ ] All tests pass (`cargo test`)
- [ ] Binary builds successfully (`cargo build --release`)
- [ ] PKGBUILDs build successfully (`makepkg -s`)
- [ ] Working directory clean (`git status`)
- [ ] SSH key configured for AUR (`ssh aur@aur.archlinux.org`)
- [ ] GitHub CLI authenticated (`gh auth status`)
- [ ] Version numbers updated di semua file
- [ ] All changes committed
- [ ] Changes pushed to main
- [ ] Tag created and pushed
- [ ] GitHub Actions completed
- [ ] GitHub release created
- [ ] AUR packages published
- [ ] Test install from AUR

---

### Common Mistakes to Avoid

❌ **Create tag sebelum push main**
✅ Push main dulu, baru create tag

❌ **Push tag, lalu commit fix**
✅ Commit fix dulu, delete tag, create tag baru

❌ **Lupa load SSH key**
✅ Run `eval "$(ssh-agent -s)" && ssh-add ~/.ssh/id_nixval` sebelum push ke AUR

❌ **Pakai escaped variables di PKGBUILD (\${pkgver})**
✅ Pakai `${pkgver}` tanpa backslash

❌ **Retag untuk PKGBUILD fixes**
✅ Tidak perlu retag, cukup push update ke AUR

---

## Quick Summary

**Release flow:**
1. Update version files
2. Run tests
3. Build binary
4. Prepare PKGBUILDs
5. Test PKGBUILDs
6. Commit changes
7. Push to main
8. Create tag
9. Push tag
10. Wait for GitHub Actions (±10 min)
11. Update declarch-bin SHA256
12. Generate .SRCINFO files
13. Publish ke AUR (3 packages)
14. Test install

**Total time: ±20-30 menit**

---

**Remember:** When in doubt, delete tag and start over. Better late than broken!
