# AUR Publishing Guide for v0.4.1

**Rilis:** 20 Januari 2026
**Versi:** 0.4.1

---

## ✅ Pre-Publish Checklist

Sebelum publish ke AUR, pastikan:

- [x] **Cargo.toml**: `version = "0.4.1"`
- [x] **PKGBUILD**: `pkgver=0.4.1`, deskripsi updated
- [x] **RELEASE.md**: Changelog v0.4.1 ditambahkan
- [x] **README.md**: Badge updated ke v0.4.1
- [x] **Git tag v0.4.1**: Dibuat
- [x] **Tests**: 141 tests passing
- [x] **Clippy**: 0 warnings (`-D warnings` strict mode)
- [x] **Build**: Release build berhasil

---

## 📋 Step-by-Step AUR Publish

### STEP 1: Push ke GitHub (BELUM dilakukan)

```bash
git push origin main
git push origin v0.4.1
```

**PENTING:** Tunggu setelah push sampai GitHub release tarball tersedia!

---

### STEP 2: Generate SHA256 Checksums

**Tunggu 30-60 detik setelah push tag**, lalu:

```bash
# Download release tarball dari GitHub
wget https://github.com/nixval/declarch/archive/refs/tags/v0.4.1/declarch-0.4.1.tar.gz

# Generate checksum
sha256sum declarch-0.4.1.tar.gz
# Output: <hash>  declarch-0.4.1.tar.gz
```

**Update PKGBUILD:**

```bash
# Edit PKGBUILD di root
nano PKGBUILD

# Ganti baris ini:
sha256sums=('SKIP')

# Dengan hash yang dihasilkan:
sha256sums('<hash-dari-wget>')
```

**Commit SHA256 update:**

```bash
git add PKGBUILD
git commit -m "chore(release): add sha256sums for v0.4.1"
git push origin main
```

---

### STEP 3: Test PKGBUILD Lokal

```bash
# Build package
makepkg -sf

# Install dan test
sudo pacman -U declarch-0.4.1-1-x86_64.pkg.tar.zst

# Cek versi
declarch --version
# Harus output: declarch 0.4.1

# Cek help
declarch --help

# Cek backends
declarch check
```

---

### STEP 4: Siapkan AUR Package

```bash
# Clone AUR repo (jika belum)
git clone ssh://aur@aur.archlinux.org/declarch.git /tmp/declarch-aur

# Atau gunakan script publish
./.aur/publish.sh 0.4.1
```

**Script akan otomatis:**
1. Copy PKGBUILD ke `/tmp/declarch-aur`
2. Commit dengan pesan "upgpkg: 0.4.1"
3. Push ke AUR

---

### STEP 5: Publish ke AUR

```bash
cd /tmp/declarch-aur

# Review dulu PKGBUILD
cat PKGBUILD

# Commit
git add PKGBUILD
git commit -m "upgpkg: 0.4.1

- Add user-defined backend support
- Backend filtering with --backend flag
- Production-ready examples for NALA, Zypper, DNF5, APT, Pacman
- All 141 tests passing
- All clippy warnings resolved"

# Push ke AUR
git push origin master
```

---

## ⏳ Tunggu AUR Processing

Setelah push ke AUR:

1. **Tunggu 5-10 menit** untuk AUR web interface update
2. **Cek di:** https://aur.archlinux.org/packages/declarch
3. **Harus muncul:**
   - Version: 0.4.1
   - Changelog terlihat di "View Changes"
   - Tarball tersedia untuk download

---

## 🧪 Post-Publish Testing

Di sistem lain (bersih dari declarch):

```bash
# Install dari AUR
paru -S declarch
# atau
yay -S declarch

# Cek versi
declarch --version

# Test basic functionality
declarch check --verbose
```

---

## 📊 File Changes untuk v0.4.1

### Root PKGBUILD Changes:

```diff
-pkgver=0.4.0
+pkgver=0.4.1
-pkgdesc="A declarative package manager for Linux (supports AUR, Flatpak, Soar)"
+pkgdesc="A declarative package manager for Linux with user-defined backend support"
```

### Cargo.toml Changes:

```diff
-version = "0.4.0"
+version = "0.4.1"
```

---

## 🚨 Troubleshooting

### Error: "Source file not found"

**Problem:** GitHub tarball belum tersedia

**Solution:**
```bash
# Tunggu beberapa menit setelah push tag
# Cek URL ini ada 404 atau tidak:
wget https://github.com/nixval/declarch/archive/refs/tags/v0.4.1/declarch-0.4.1.tar.gz
```

### Error: "integrity check failed"

**Problem:** SHA256 checksum salah

**Solution:**
```bash
# Regenerate checksum
makepkg -g

# Update PKGBUILD sha256sums line dengan output baru
```

### Error: "Permission denied (publickey)"

**Problem:** SSH key untuk AUR belum configured

**Solution:**
```bash
# Cek SSH key
ls -la ~/.ssh/*.pub

# Test connection
ssh aur@aur.archlinux.org

# Jika failed, setup SSH key:
ssh-keygen -t ed25511 -C "nixval@aur" -f ~/.ssh/aur_key -N ""
cat ~/.ssh/aur_key.pub
# Copy → https://aur.archlinux.org/account → SSH Public Keys
```

---

## 📝 Setelah Publish Sukses

1. **Update AUR comment** (optional):
   - Cek: https://aur.archlinux.org/packages/declarch
   - Add comment dengan highlights v0.4.1

2. **Update wiki/documentation** (if needed):
   - Arch Wiki page (jika ada)
   - GitHub wiki

3. **Announce** (optional):
   - Reddit: r/archlinux
   - GitHub Discussions
   - Matrix/Discord channels

---

## 🎯 Summary

**Command sequence:**

```bash
# 1. Push ke GitHub (DO THIS FIRST!)
git push origin main
git push origin v0.4.1

# 2. Generate SHA256 (TUNGGU 30-60 DETIK!)
wget https://github.com/nixval/declarch/archive/refs/tags/v0.4.1/declarch-0.4.1.tar.gz
sha256sum declarch-0.4.1.tar.gz

# 3. Update PKGBUILD dengan sha256sums
nano PKGBUILD
git add PKGBUILD
git commit -m "chore(release): add sha256sums for v0.4.1"
git push origin main

# 4. Test PKGBUILD
makepkg -sf
sudo pacman -U declarch-0.4.1-1-x86_64.pkg.tar.zst
declarch --version

# 5. Publish ke AUR
./.aur/publish.sh 0.4.1

# 6. Tunggu 5-10 menit, cek:
# https://aur.archlinux.org/packages/declarch
```

**All clear!** ✅
