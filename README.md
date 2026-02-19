# declarch

Declarch adalah wrapper deklaratif untuk banyak package manager.

Kamu deklarasikan paket di file KDL, lalu jalankan `declarch sync`.

## WARNING: v0.8.0 has BREAKING CHANGES

Kalau upgrade dari versi lama, expect perubahan syntax/workflow.

Sebelum upgrade:

```bash
cp -r ~/.config/declarch ~/.config/declarch.backup
```

Untuk cek path config/state yang benar di OS kamu:

```bash
declarch info --doctor
```

## Core flow (yang wajib dipahami)

1. Tulis konfigurasi paket (KDL).
2. Preview perubahan.
3. Apply.

```bash
declarch --dry-run sync
declarch sync
```

## Instalasi

### Arch Linux (AUR)

```bash
paru -S declarch
# atau
yay -S declarch
```

### Linux/macOS (install script)

```bash
curl -sSL https://raw.githubusercontent.com/nixval/declarch/main/install.sh | sh
```

### Windows (PowerShell, experimental alpha)

```powershell
irm https://raw.githubusercontent.com/nixval/declarch/main/install.ps1 | iex
```

Verifikasi:

```bash
declarch --version
declarch --help
declarch info --doctor -v
```

## First setup (beginner path)

### 1) Init config

```bash
declarch init
```

### 2) Adopt backend sesuai OS

```bash
# Arch
declarch init --backend aur,paru,yay,pacman

# Debian/Ubuntu
declarch init --backend apt,nala

# Fedora/RHEL
declarch init --backend dnf5

# SUSE
declarch init --backend zypper

# macOS
declarch init --backend brew

# lihat backend/module dari registry
declarch init --list backends
declarch init --list modules
```

### 3) Tambah paket (declarative-first)

Contoh `~/.config/declarch/modules/base.kdl`:

```kdl
pkg {
    aur { bat fzf ripgrep }
    npm { typescript }
}
```

Lalu:

```bash
declarch --dry-run sync
declarch sync
```

### 4) Shortcut cepat via install (opsional)

```bash
declarch install aur:bat aur:fzf aur:ripgrep
declarch install npm:typescript
# atau pakai backend tunggal:
declarch install bat fzf ripgrep --backend aur
```

## Contoh konfigurasi yang sering dipakai

### A) Minimal portable config

`~/.config/declarch/declarch.kdl`

```kdl
imports {
    "modules/base.kdl"
}
```

`~/.config/declarch/modules/base.kdl`

```kdl
pkg {
    pacman { git curl }
    flatpak { org.mozilla.firefox }
    npm { pnpm }
}
```

### B) Pisah module per kebutuhan

`declarch.kdl`

```kdl
imports {
    "modules/base.kdl"
    "modules/dev.kdl"
    "modules/apps.kdl"
}
```

`modules/dev.kdl`

```kdl
pkg {
    aur { neovim tmux }
    npm { typescript eslint prettier }
}
```

`modules/apps.kdl`

```kdl
pkg {
    flatpak { org.telegram.desktop com.spotify.Client }
}
```

### C) Profile + host layering (opsional)

```kdl
profile "work" {
    pkg {
        npm { @angular/cli }
    }
}

host "laptop-1" {
    pkg {
        flatpak { com.discordapp.Discord }
    }
}
```

Pakai saat sync:

```bash
declarch sync --profile work
declarch sync --host laptop-1
```

### D) Hooks (opt-in, aman by default)

```kdl
hooks {
    pre-sync "echo before-sync"
    post-sync "echo after-sync"
}

experimental {
    "enable-hooks"
}
```

Eksekusi hooks:

```bash
declarch sync --hooks
```

### E) Policy block (kontrol perilaku sync/lint)

```kdl
policy {
    protected "linux" "systemd"
    orphans "ask"
    require_backend "true"
    forbid_hooks "false"
    on_duplicate "warn"
    on_conflict "warn"
}
```

## Command yang paling sering dipakai

```bash
declarch sync
declarch --dry-run sync
declarch sync update
declarch sync prune
declarch sync cache
declarch sync upgrade
declarch search firefox
declarch info --doctor
declarch info --list --scope unmanaged
declarch lint --mode validate
declarch edit mymodule --create
```

## Update policy

- Jika install via package manager (AUR/Homebrew/dll), update via package manager itu.
- Jika install via script/manual binary, bisa pakai `declarch self-update`.

## Dokumentasi

- https://nixval.githu.io/declarch/

## License

MIT - lihat `LICENSE`.
