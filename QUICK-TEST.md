# 🚀 Quick Test Start

Cara cepat mulai testing backend system baru:

## 1️⃣ Test npm Backend (Paling Mudah)

```bash
# Test dengan config siap pakai
declarch --config ~/.config/declarch/tests/test-npm-only.kdl check

# Preview install
declarch --config ~/.config/declarch/tests/test-npm-only.kdl sync --dry-run

# Install untuk beneran
declarch --config ~/.config/declarch/tests/test-npm-only.kdl sync

# Verify dengan native command
npm list -g --depth=0 | grep prettier
```

---

## 2️⃣ Test Mixed Backends

```bash
# Cek config
declarch --config ~/.config/declarch/tests/test-mixed.kdl check

# Preview
declarch --config ~/.config/declarch/tests/test-mixed.kdl sync --dry-run

# Install
declarch --config ~/.config/declarch/tests/test-mixed.kdl sync
```

---

## 3️⃣ Test Unavailable Backend

```bash
# Harus keluar warning:
# ⚠ Skipping 1 package(s) from unavailable backends.

declarch --config ~/.config/declarch/tests/test-unavailable.kdl sync --dry-run
```

---

## 4️⃣ Test Inline Syntax

```bash
declarch --config ~/.config/declarch/tests/test-inline.kdl sync --dry-run
```

---

## 5️⃣ Test Manual Config

```bash
# Buat config sendiri
cat > /tmp/my-test.kdl << 'EOF'
meta {
    description "My test"
}

packages:npm {
    typescript
}

packages:pip {
    pipx
}

packages:cargo {
    sccache
}
EOF

# Test
declarch --config /tmp/my-test.kdl sync --dry-run
```

---

## 🔍 Verify Installation

```bash
# npm
npm list -g --depth=0

# pip
pip list

# cargo
cargo install --list

# Cek state declarch
cat ~/.local/state/declarch/state.json | jq '.packages | keys'
```

---

## 📁 File Test yang Tersedia

```
~/.config/declarch/tests/
├── test-npm-only.kdl       # Hanya npm packages
├── test-mixed.kdl          # npm + pip + cargo
├── test-inline.kdl         # Inline prefix syntax
└── test-unavailable.kdl    # Backend tidak available
```

---

## ⚡ Commands Penting

| Command | Description |
|---------|-------------|
| `--dry-run` | Preview tanpa eksekusi |
| `--target <backend>` | Sync backend tertentu saja |
| `--prune` | Remove packages tidak di config |
| `-v` | Verbose output untuk debug |

---

## ✅ Test Flow Recommendation

1. **Mulai dengan test-npm-only.kdl** → Paling simple
2. **Lanjut test-mixed.kdl** → Multiple backends
3. **Coba test-inline.kdl** → Syntax variations
4. **Test unavailable.kdl** → Error handling
5. **Buat config sendiri** → Real use case

---

## 🐛 Troubleshooting

### Config tidak terbaca?
```bash
declarch --config <path> check
```

### Package tidak terinstall?
```bash
declarch --config <path> sync -v  # Verbose mode
```

### State file corrupt?
```bash
# Restore dari backup
cp ~/.local/state/declarch/state.json.backup.1 ~/.local/state/declarch/state.json
```

### Cek apa yang di-track declarch?
```bash
declarch info
cat ~/.local/state/declarch/state.json | jq '.packages | keys'
```

---

Happy testing! 🎉
