# 🧪 Testing Guide: Generic Backend System

Panduan lengkap untuk testing fitur baru backend system.

## 📋 Daftar Use Case

### 1. **npm Backend - Node.js Global Packages**

#### Test Install Package
```bash
# Buat config test
cat > ~/.config/declarch/test-npm.kdl << 'EOF'
meta {
    description "Test npm backend"
    author "testing"
}

packages:npm {
    typescript
    prettier
    eslint
}
EOF

# Cek config
declarch check

# Preview install
declarch sync --dry-run

# Install untuk beneran
declarch sync

# Cek apakah terinstall
npm list -g --depth=0 | grep -E "typescript|prettier|eslint"
```

#### Test Remove Package
```bash
# Edit config, hapus salah satu package
# Lalu jalankan:
declarch sync --dry-run  # Preview
declarch sync --prune     # Remove
```

---

### 2. **pip Backend - Python Packages**

#### Test Install
```kdl
// ~/.config/declarch/test-pip.kdl
packages:pip {
    ruff
    black
    pytest
}
```

```bash
# Cek dan sync
declarch check
declarch sync --dry-run

# Install
declarch sync

# Verify
pip list | grep -E "ruff|black|pytest"
```

---

### 3. **cargo Backend - Rust Crates**

#### Test Install
```kdl
// ~/.config/declarch/test-cargo.kdl
packages:cargo {
    ripgrep
    fd-find
    eza
    bat
}
```

```bash
declarch check
declarch sync --dry-run
declarch sync

# Verify
cargo install --list | grep -E "ripgrep|fd|eza|bat"
```

---

### 4. **Mixed Backends - Multiple Language Ecosystems**

#### Test Combined Config
```kdl
// ~/.config/declarch/test-mixed.kdl
meta {
    description "Test multiple backends simultaneously"
    author "testing"
}

// Node.js packages
packages:npm {
    typescript
}

// Python packages
packages:pip {
    pipx
}

// Rust tools
packages:cargo {
    sccache
}

// AUR packages (Arch only)
packages:aur {
    bat-catppuccin
}
```

```bash
# Cek semua backend terdeteksi
declarch check

# Preview - harusnya lihat packages dari 3-4 backend
declarch sync --dry-run

# Install semua
declarch sync
```

---

### 5. **Inline Prefix Syntax**

#### Test Format Berbeda
```kdl
// ~/.config/declarch/test-inline.kdl

// Semua dalam satu block dengan prefix
packages {
    // AUR (default)
    neovim

    // Dengan prefix eksplisit
    aur:python-pynvim
    npm:prettier
    pip:colorama
    cargo:tealdeer

    // Bisa juga nested
    aur {
        fzf
    }
}
```

```bash
declarch check
declarch sync --dry-run
```

---

### 6. **Test State Management**

#### Cek State File
```bash
# Lihat state file
cat ~/.local/state/declarch/state.json | jq '.packages | keys'

# Cek spesifik npm package
cat ~/.local/state/declarch/state.json | jq '.packages | to_entries | map(select(.key | contains("npm")))'

# Cek spesifik pip package
cat ~/.local/state/declarch/state.json | jq '.packages | to_entries | map(select(.key | contains("pip")))'
```

---

### 7. **Test Adopts (Package Already Installed)**

#### Install Manual, Lalu Adopt
```bash
# Install npm package manual
npm install -g json-server

# Tambah ke config
cat > ~/.config/declarch/test-adopt.kdl << 'EOF'
packages:npm {
    json-server  // Sudah terinstall manual
}
EOF

# Sync - harusnya "adopt" bukan install
declarch sync --dry-run

# Cek state - sekarang tracked
declarch info
```

---

### 8. **Test Pruning (Remove Unused Packages)**

#### Hapus dari Config, Lalu Prune
```bash
# Awal: 3 packages
# Edit config, tinggalkan 1 package
declarch sync --dry-run --prune  # Lihat apa yang akan di-remove
declarch sync --prune            # Execute remove
```

---

### 9. **Test Unavailable Backend**

#### Backend yang Tidak Ada di System
```kdl
// ~/.config/declarch/test-unavailable.kdl

// brew tidak ada di Linux
packages:brew {
    node
}

// npm ada
packages:npm {
    typescript
}
```

```bash
# Harusnya warning:
# ⚠ Skipping 1 package(s) from unavailable backends.
# ℹ   Skipping node (backend 'brew' not available)

declarch sync --dry-run
```

---

### 10. **Test Module Import**

#### Backend Packages dalam Modules
```bash
# Buat module
cat > ~/.config/declarch/modules/dev-tools.kdl << 'EOF'
// Development tools module
meta {
    description "Language-agnostic dev tools"
    author "test"
}

packages:npm {
    prettier
    eslint
}

packages:pip {
    ruff
    black
}
EOF

# Import di main config
cat >> ~/.config/declarch/declarch.kdl << 'EOF'
imports {
    "modules/dev-tools"
}
EOF

# Test
declarch check
declarch sync --dry-run
```

---

### 11. **Test Info Command**

#### Lihat Managed Packages per Backend
```bash
# Setelah install beberapa backend
declarch info

# Output harusnya breakdown:
# Total Managed X
#     • AUR/Repo:  X
#     • Flatpak:   X
#     • Npm:       X  ← NEW
#     • Pip:       X  ← NEW
#     • Cargo:     X  ← NEW
```

---

### 12. **Test Version Updates**

#### Update Package Version
```bash
# Awal: install versi tertentu
# Lalu upgrade manual: npm install -g prettier@latest

# Sync - harusnya detect version drift
declarch sync --dry-run

# Cek state terupdate
cat ~/.local/state/declarch/state.json | jq '.packages["npm:prettier"]'
```

---

### 13. **Test Error Handling**

#### Package Manager Tidak Installed
```bash
# Uninstall npm sementara
# sudo pacman -R npm  (Arch)

# Coba sync npm packages
declarch sync --dry-run

# Harusnya skip npm packages
```

---

### 14. **Test Specific Backend Targeting**

#### Sync Backend Tertentu Saja
```bash
# Hanya sync npm packages
declarch sync --target npm --dry-run

# Hanya sync pip
declarch sync --target pip --dry-run

# Hanya sync aur
declarch sync --target aur --dry-run
```

---

## 🔍 Debugging Commands

### Cek Package Terinstall per Backend
```bash
# npm
npm list -g --depth=0 --json | jq '.dependencies | keys'

# pip
pip list --format=json | jq -r '.[].name'

# cargo
cargo install --list

# AUR/pacman
pacman -Qqm
```

### Cek State File
```bash
# Lihat semua state
cat ~/.local/state/declarch/state.json | jq '.'

# Lihat key tertentu
cat ~/.local/state/declarch/state.json | jq '.packages | keys' | grep npm

# Lihat detail package
cat ~/.local/state/declarch/state.json | jq '.packages["npm:prettier"]'
```

### Backup & Restore State
```bash
# Backup sebelum testing
cp ~/.local/state/declarch/state.json ~/.local/state/declarch/state.json.backup

# Restore jika ada masalah
cp ~/.local/state/declarch/state.json.backup ~/.local/state/declarch/state.json
```

---

## 📝 Test Checklist

- [ ] **npm**: Install package
- [ ] **npm**: Remove package (dengan prune)
- [ ] **npm**: Adopt existing package
- [ ] **pip**: Install package
- [ ] **cargo**: Install package
- [ ] **Mixed**: Multiple backends in one config
- [ ] **Inline**: Prefix syntax (`npm:package`)
- [ ] **Module**: Backend packages dalam imported module
- [ ] **Unavailable**: Skip backend yang tidak ada
- [ ] **Info**: Command shows backend breakdown
- [ ] **State**: Correctly tracked per backend
- [ ] **Prune**: Remove untracked packages safely
- [ ] **Target**: Sync specific backend only

---

## 🚨 Quick Test Commands

```bash
# 1. Cek health system
declarch info
declarch check

# 2. Preview changes
declarch sync --dry-run

# 3. Apply changes
declarch sync

# 4. Update system + sync
declarch sync -u

# 5. Remove unused packages
declarch sync --prune

# 6. Edit config
declarch edit

# 7. Cek specific backend
declarch sync --target npm --dry-run
```

---

## 💡 Tips

1. **Selalu pakai --dry-run dulu** untuk preview
2. **Backup state file** sebelum testing berat
3. **Test satu backend dulu**, baru mixed
4. **Cek dengan native command** untuk verify install success
5. **Gunakan verbose mode** untuk debug: `-v` atau `--verbose`

Selamat testing! 🎉
