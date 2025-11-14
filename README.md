# 🌱 **declarch**

<p align="center">
  <strong>A declarative package manager for Arch Linux — powered by Rust, inspired by Nix.</strong><br>
  Make your Arch setup reproducible, modular, and clean.
</p>

<p align="center">
  <img alt="License" src="https://img.shields.io/badge/license-MIT-blue">
  <img alt="Build" src="https://img.shields.io/badge/status-alpha-orange">
  <img alt="Arch" src="https://img.shields.io/badge/arch-linux-blue">
  <img alt="Rust" src="https://img.shields.io/badge/built_with-rust-orange">
</p>

---

## 🌟 What Is `declarch`?

Arch Linux is powerful, but its package management is **fully imperative** — you install things manually, forget what you installed, and eventually the system becomes a museum of old packages.

`declarch` brings **declarative package management** to Arch, without trying to replace pacman or introduce a new filesystem.
You write *what you want*, and `declarch` ensures your system matches it.

Think of it as:

> 🧠 *“Nix-style reproducibility, but the Arch way.”*

---

## ✨ Features

* **Declarative system state** – control everything through KDL files.
* **KDL configs** – clean syntax, zero indentation nightmares.
* **Modular design** – split configs into `gaming.kdl`, `shell.kdl`, `dev.kdl`, etc.
* **Per-host configs** – different packages for laptop, desktop, server.
* **Safe pruning** – only removes packages managed by `declarch`, never the whole system.
* **Conflict detection** – avoid enabling incompatible modules.
* **Version pinning warnings** – get notified if versions drift.
* **AUR support** – works seamlessly with `paru` or other helpers.

---

# 🚀 Installation

## Option 1 — Install Script (recommended)

Downloads the latest release binary and installs it to `/usr/local/bin/`.

```bash
curl -sSL https://raw.githubusercontent.com/nixval/declarch/main/install.sh | bash
```

---

## Option 2 — Build From Source

```bash
git clone https://github.com/nixval/declarch.git
cd declarch
cargo build --release
sudo cp target/release/declarch /usr/local/bin/
```

---

# 📁 Initial Setup (Required)

Before using `declarch`, create the config directory:

```bash
mkdir -p ~/.config/declarch/modules
mkdir -p ~/.config/declarch/hosts
```

---

## 1. `config.kdl` (main entrypoint)

```kdl
host "your_hostname_here"
enabled_modules "base"
```

> Replace `"your_hostname_here"` with the output of `hostname`.

---

## 2. `hosts/<hostname>.kdl`

For machine-specific packages:

```kdl
description "Machine-specific packages"
packages zsh
```

---

## 3. `modules/base.kdl`

Your global packages:

```kdl
description "Base packages"
packages git vim ripgrep
```

---

# 🔧 First Sync

```bash
declarch sync
```

`declarch` will initialize state tracking and install the packages you declared.

---

# 🧠 Usage Overview

Once setup is done, the workflow becomes extremely simple:

1. Edit KDL files
2. Run `declarch sync`

That’s it.

---

## 📦 Module Management

List all modules:

```bash
declarch module list
```

Enable one:

```bash
declarch module enable gaming
```

Disable:

```bash
declarch module disable gaming
```

---

## 🔄 Synchronization

Normal sync:

```bash
declarch sync
```

Sync + prune unused packages:

```bash
declarch sync --prune
```

Only packages managed by `declarch` will ever be pruned — safe for daily use.

---

# 🧩 Advanced Examples

## Excluding packages on specific machines

```kdl
packages fastfetch
exclude neofetch
```

---

## Module conflict safety

```kdl
description "Hyprland compositor"
packages hyprland
conflicts sway
```

---

## Version pinning with warnings

```kdl
packages "git=1.0.0" vim
```

---

# 📜 License

MIT — free to use, modify, hack, and enjoy.

---
