# EndeavourOS Setup Guide

Panduan setup environment untuk development dan publish AUR package di EndeavourOS.

---

## 1. System Update

Setelah fresh install EndeavourOS:

```bash
# Update system
sudo pacman -Syu

# Reboot if kernel updated
sudo reboot
```

---

## 2. Install Development Tools

```bash
# Base development tools
sudo pacman -S --needed base-devel git

# Rust toolchain (untuk build declarch)
sudo pacman -S rust

# Atau install rustup (recommended)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
```

---

## 3. Install AUR Helper

**Option 1: paru (Recommended)**

```bash
# Install dependencies
sudo pacman -S --needed base-devel

# Clone dan build paru
git clone https://aur.archlinux.org/paru.git
cd paru
makepkg -si

cd ..
rm -rf paru
```

**Option 2: yay**

```bash
# Install dependencies
sudo pacman -S --needed base-devel

# Clone dan build yay
git clone https://aur.archlinux.org/yay.git
cd yay
makepkg -si

cd ..
rm -rf yay
```

---

## 4. Setup Git

```bash
# Configure git
git config --global user.name "Your Name"
git config --global user.email "your@email.com"

# Generate SSH key (jika belum ada)
ssh-keygen -t ed25519 -C "your@email.com"

# Copy public key
cat ~/.ssh/id_ed25519.pub
```

**Add SSH key ke:**
- GitHub: https://github.com/settings/keys
- AUR: https://aur.archlinux.org/account

---

## 5. Install AUR Package Building Tools

```bash
# Additional tools untuk PKGBUILD development
sudo pacman -S \
    namcap \
    pkgstats
```

**Tools explanation:**
- `namcap` - Check PKGBUILD untuk common issues
- `pkgstats` - Statistics untuk AUR packages

---

## 6. Test Build declarch

```bash
# Clone repo
git clone https://github.com/nixval/declarch.git
cd declarch

# Build release
cargo build --release

# Test binary
./target/release/declarch --version
./target/release/declarch info
```

---

## 7. Test PKGBUILD

Sebelum publish ke AUR, test PKGBUILD locally:

```bash
# Create test directory
mkdir -p /tmp/declarch-test
cd /tmp/declarch-test

# Copy PKGBUILD
cp /path/to/declarch/PKGBUILD .

# Download source
makepkg -s

# Check package with namcap
namcap PKGBUILD *.pkg.tar.zst

# Install package untuk test
sudo pacman -U declarch-*.pkg.tar.zst

# Test installed binary
declarch --version

# Uninstall test package
sudo pacman -R declarch
```

---

## 8. Generate SHA256 Checksums

**Untuk PKGBUILD (source build):**

```bash
# Download source
makepkg -g

# Copy output sha256sums ke PKGBUILD
# Atau langsung update:
makepkg -g >> PKGBUILD
```

**Untuk PKGBUILD-bin (binary release):**

```bash
# Download binary dari GitHub release
wget https://github.com/nixval/declarch/releases/download/v0.4.0/declarch

# Generate sha256sum
sha256sum declarch

# Output format:
# <hash>  declarch

# Copy hash ke PKGBUILD-bin sha256sums=()
```

---

## 9. Setup AUR SSH Access

```bash
# Test SSH ke AUR
ssh aur@aur.archlinux.org

# Should output:
# Welcome to the Arch User Repository
```

**Kalau gagal:**
1. Pastikan SSH key sudah di-generate
2. Pastikan public key ditambah di https://aur.archlinux.org/account
3. Tunggu beberapa menit untuk propagasi

---

## 10. AUR Publish Workflow

```bash
# 1. Update version di PKGBUILD
vim PKGBUILD  # Edit pkgver

# 2. Generate sha256sums
cd /tmp
mkdir test-declarch
cd test-declarch
cp /path/to/declarch/PKGBUILD .
makepkg -g >> PKGBUILD

# 3. Test build
makepkg -si

# 4. Test install
paru -Q declarch  # Check installed

# 5. Uninstall test package
sudo pacman -R declarch

# 6. Publish ke AUR
cd /path/to/declarch
./.aur/publish.sh 0.4.0

# Script akan:
# - Prepare packages di /tmp/
# - Generate .SRCINFO
# - Show preview
# - Ask: "Publish to AUR? (y/N)"
# - Push ke AUR via git
```

---

## 11. Useful Commands

### Check Package Info

```bash
# Check installed package
paru -Qi declarch

# Check package files
paru -Ql declarch

# Check package from AUR web
paru -Si declarch
```

### Clean Build Cache

```bash
# Clean pacman cache
sudo paccache -ruk0

# Clean cargo cache
cargo clean

# Clean makepkg build directory
rm -rf /tmp/declarch-*
```

### Debug PKGBUILD

```bash
# Run makepkg dengan verbose
makepkg -sv

# Check PKGBUILD syntax
namcap PKGBUILD

# Install with --asdeps to mark as dependency
sudo pacman -U --asdeps *.pkg.tar.zst
```

---

## 12. Common Issues

### Issue: makepkg failed - missing cargo

**Solution:**
```bash
sudo pacman -S rust
# atau
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

---

### Issue: permission denied /usr/bin

**Solution:**
```bash
# Use sudo
sudo pacman -U declarch-*.pkg.tar.zst

# Atau install to user directory (not recommended)
makepkg -si --packager "Your Name"
```

---

### Issue: signature is invalid

**Solution:**
```bash
# Remove SKIP from sha256sums
# Generate proper sha256sums:
makepkg -g >> PKGBUILD
```

---

### Issue: AUR push failed - permission denied

**Solution:**
```bash
# Check if you're maintainer
# Go to: https://aur.archlinux.org/packages/declarch

# Check SSH key
ssh aur@aur.archlinux.org

# Re-add SSH key if needed
cat ~/.ssh/id_ed25519.pub
# Paste ke: https://aur.archlinux.org/account
```

---

## 13. Verification Checklist

Sebelum publish ke AUR, pastikan:

- [ ] Cargo.toml version = PKGBUILD pkgver
- [ ] Binary works: `./target/release/declarch --version`
- [ ] PKGBUILD builds: `makepkg -si`
- [ ] namcap check passes: `namcap PKGBUILD *.pkg.tar.zst`
- [ ] sha256sums generated (bukan SKIP)
- [ ] SSH key works: `ssh aur@aur.archlinux.org`
- [ ] Tested fresh install di clean chroot (optional)

---

## 14. Quick Reference

**Full setup (one-liner):**

```bash
# Update system
sudo pacman -Syu

# Install dev tools
sudo pacman -S --needed base-devel git rust

# Install AUR helper
git clone https://aur.archlinux.org/paru.git
cd paru && makepkg -si && cd .. && rm -rf paru

# Build declarch
cargo build --release
./target/release/declarch --version
```

**Publish AUR:**

```bash
# Generate sha256sums
cd /tmp && mkdir test && cd test
cp /path/to/declarch/PKGBUILD .
makepkg -g >> PKGBUILD

# Test build
makepkg -si

# Publish
cd /path/to/declarch
./.aur/publish.sh 0.4.0
```

---

## 15. Resources

- **EndeavourOS Wiki**: https://discovery.endeavouros.com/
- **Arch Wiki**: https://wiki.archlinux.org/
- **AUR Guidelines**: https://wiki.archlinux.org/title/Arch_User_Repository
- **PKGBUILD Example**: https://wiki.archlinux.org/title/PKGBUILD
- **makepkg man page**: `man makepkg`

---

Last updated: 2025-01-13
