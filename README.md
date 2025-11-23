
# 🌱 **declarch**

<p align="center">
<strong>A declarative package manager for Arch Linux — powered by Rust.\</strong\><br>
Inspired by Nix workflow, built for the chaotic reality of Arch.
</p>


<p align="center">
  <img alt="License" src="https://img.shields.io/badge/license-MIT-blue">
  <img alt="Build" src="https://img.shields.io/badge/status-alpha-orange">
  <img alt="Arch" src="https://img.shields.io/badge/arch-linux-blue">
  <img alt="Rust" src="https://img.shields.io/badge/built_with-rust-orange">
</p>

-----

## 🧠 The Philosophy

Arch Linux is fantastic, but its package management is **imperative**. You run `pacman -S git`, and then you forget about it. Over time, your system becomes a "museum" of forgotten packages, orphans, and drift.

```kdl
// ~/.config/hypr/hyprland.decl

packages {
  hyprland 
  hyprlock
  hypridle 
  quickshell
  noctalia-shell
  //till bunch of other required and optional packages tracked
}
```
then just simply

```bash
declarch sync
```
then just share it with your own dotfiles to anyone that using arch base distro

**declarch** imposes a **Declarative Layer** on top of Pacman/AUR without replacing them.

1.  **Intent vs. State:** You declare *what* you want in a `.kdl` file. `declarch` ensures your system matches that state.
2.  **Adoption, Not Reinstallation:** If you declare `vim` and it's already installed manually, `declarch` simply "adopts" it into its state file.
3.  **Performance:** Uses smart batching to check hundreds of packages instantly.
4.  **Safe Pruning:** Only removes packages that it *knows* it manages.

-----

## ✨ Key Features

  * **Declarative Config:** Uses the clean, readable **KDL** syntax (`.kdl`).
  * **Recursive Imports:** Structure your config cleanly (e.g., `import "modules/gaming"`).
  * **Multi-Backend:** Supports **Repo (Pacman)**, **AUR (Paru/Yay)**, and **Flatpak**.
  * **Partial Sync:** Sync only specific modules or packages with `--target`.
  * **Smart Sync:**
      * **Install:** Missing packages.
      * **Adopt:** Existing packages (zero-cost).
      * **Prune:** Packages removed from config (optional strict mode).
  * **Drift Detection:** Detects if package versions in system differ from state.

-----

## 🚀 Installation
### Option 1: Install from AUR

```bash
paru -S declarch-bin
```

or compile manually

```bash 
paru -S declarch
```

### Option 2: Install Script

Downloads the latest binary and sets up the environment.

```bash
curl -sSL https://raw.githubusercontent.com/nixval/declarch/main/install.sh | bash
```

### Option 2: Build from Source

Requirements: `cargo`, `git`, `pacman`.

```bash
git clone https://github.com/nixval/declarch.git
cd declarch
cargo install --path .
```

-----

## 📁 Getting Started

Initialize your configuration directory. This respects XDG standards (`~/.config/declarch`).

```bash
declarch init
```

This creates a default `declarch.decl` entry point.

### Anatomy of `declarch.kdl`

The syntax uses **KDL**. Quotes are optional for simple names.

```kdl
// ~/.config/declarch/declarch.kdl

// 1. Imports: Load other declarative files
imports {
    "modules/core.kdl"
    "modules/dev.kdl"
}

// 2. Packages: Define what you want installed
packages {
    // Native / AUR packages (auto-detected)
    git
    neovim
    zsh
    
    // Flatpak packages (use prefix) **make sure no space type**
    flatpak:com.obsproject.Studio
    flatpak:spotify
}

// 3. Excludes: Block specific packages even if imported by modules
excludes {
    nano
    vi
}
```
-----

## 🛠️ Usage Workflow

The workflow is simple: **Edit** -\> **Sync**.

### 1\. The Magic Command
Update system, sync packages, and remove unlisted ones:
```bash
declarch sync -u --prune
```

### 2\. Targeted Sync (Partial)
Only sync packages related to "gaming" (matches module filename or package name):
```bash
declarch sync --target gaming
```
*Note: Pruning is automatically disabled in targeted mode for safety.*

### 3\. CI/CD / Automation
Run without confirmation prompts:
```bash
declarch sync --noconfirm
```

### 4\. Command Reference
| Command | Description |
| :--- | :--- |
| `declarch init` | Create initial configuration. |
| `declarch check` | Validate syntax and check for duplicates. |
| `declarch info` | Show managed packages and stats. |
| `declarch sync` | The main workhorse. See flags below. |

### Sync Flags
| Flag | Description |
| :--- | :--- |
| `-u` / `--update` | Run `paru -Syu` before syncing. |
| `--dry-run` | Preview changes without doing anything. |
| `--prune` | **Strict Mode.** Remove managed packages not in config. |
| `--target <NAME>` | Sync only specific package or module scope. |
| `--noconfirm` | Skip package manager prompts (CI/CD). |
| `-f` / `--force` | Force operations (bypass safety checks). |

## 💡 Why KDL?

We chose [KDL](https://kdl.dev/) because it's designed for configuration, not data serialization.

  * **VS JSON:** Comments are supported\! `// like this`.
  * **VS YAML:** No whitespace/indentation anxiety.
  * **VS TOML:** Better support for nested hierarchies (blocks).

-----

## ⚠️ Safety First

`declarch` keeps its state in `~/.local/state/declarch/state.json`.

  * If you delete a package from your `.kdl` file, `declarch` will **NOT** remove it from your system unless you run `sync --prune`.
  * It automatically creates a backup of the state file before every write operation.

-----

## 🤝 Contributing

Pull requests are welcome\! This project is written in **Rust**.
Check the `src/` folder for the codebase. The core logic resides in `src/core/resolver.rs`.

1.  Fork it
2.  Create your feature branch (`git checkout -b feature/amazing-feature`)
3.  Commit your changes (`git commit -m 'Add some amazing feature'`)
4.  Push to the branch (`git push origin feature/amazing-feature`)
5.  Open a Pull Request

-----

**License:** MIT
