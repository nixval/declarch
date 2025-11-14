# declarch

A declarative package management CLI for Arch Linux, inspired by Nix and built with Rust.

## Philosophy

`declarch` (Declarative Arch) aims to bring the power and convenience of declarative package management (like Nix) to the rolling-release ecosystem of Arch Linux.

Instead of manually running `paru -S ...` and forgetting which packages you've installed, `declarch` allows you to define your **entire system** (packages, modules, and per-machine configurations) in simple, human-readable KDL text files.

`declarch` acts as your "single source of truth." It ensures your system always matches the state you defined in your configuration.

## Features

* **Declarative:** Define your desired state; let `declarch` make it happen.
* **KDL Syntax:** Uses [KDL](https://kdl.dev/) for clean, minimal config files that are immune to YAML's indentation errors.
* **Modular:** Organize your configuration into reusable modules (e.g., `gaming`, `development`, `shell`).
* **Host-Specific:** Automatically apply different packages based on the machine (e.g., `hosts/desktop.kdl` vs. `hosts/laptop.kdl`).
* **Stateful & Safe Pruning:** `declarch` tracks the packages it manages. Running `--prune` will only remove packages *you* decided you no longer want, not your entire system.
* **Conflict Detection:** Automatically prevents you from enabling mutually exclusive modules (like `hyprland` and `sway`).
* **Version Checking:** Warns you if an installed package doesn't match a version you pinned in your configuration.
* **AUR Support:** Fully integrated with `paru` (or other AUR helpers) by default.

---

## 1. Installation

There are two ways to install `declarch`.

### Option 1: Install Script (Recommended)

This script will automatically download the latest binary release from GitHub, set its permissions, and move it to `/usr/local/bin/`.

```bash
curl -sSL [https://raw.githubusercontent.com/nixval/declarch/main/install.sh](https://raw.githubusercontent.com/nixval/declarch/main/install.sh) | bash
```

Option 2: Build from Source (Manual)

If you prefer to build it manually, you will need the Rust toolchain (cargo).

```bash
# 1. Clone the repository
git clone [https://github.com/nixval/declarch.git](https://github.com/nixval/declarch.git)
cd declarch

# 2. Build the optimized release binary
cargo build --release

# 3. Move the binary to your path
sudo cp target/release/declarch /usr/local/bin/
```

2. Initial Setup (Required)

After installation, declarch won't work until you create its configuration files. By default, declarch looks for its configuration in ~/.config/declarch/.

Here are the steps to create a minimal configuration structure.

Step 1: Create Directories

Run this in your terminal to create the required directory structure:
```bash
mkdir -p ~/.config/declarch/modules
mkdir -p ~/.config/declarch/hosts
```
Step 2: Create Configuration Files

Create these three essential files.

1. ~/.config/declarch/config.kdl (Main Config File) This is the main entrypoint. declarch reads this file first.
Code snippet

```kdl
// Define your host's name. This MUST match
// the filename in the 'hosts/' folder.
host "your_hostname_here"

// List which modules you want to enable.
enabled_modules "base"
```
(You can get your_hostname_here by running the hostname command)

2. ~/.config/declarch/hosts/your_hostname_here.kdl (Host File) Rename your_hostname_here.kdl to match your actual hostname (e.g., valiE.kdl). This file is for hardware-specific packages.
Code snippet

```kdl
// ~/.config/declarch/hosts/valiE.kdl
description "Packages specific to the valiE machine"
packages zsh
```

3. ~/.config/declarch/modules/base.kdl (Your First Module) This is your base module, containing packages you want on all your machines.
Code snippet
```kdl
// ~/.config/declarch/modules/base.kdl
description "Universal base packages"
packages git vim ripgrep
```
Step 3: Run Your First Sync

You are now ready. Run your first sync. declarch will read these files, see that git, vim, ripgrep, and zsh are not in its state file, and install them.

```bash
declarch sync
```

3. Usage & Core Concepts

Managing your system is now just a matter of editing KDL files and running sync.

File Concepts

    config.kdl: The main file. It controls which host is active and which enabled_modules to load.

    hosts/*.kdl: Loaded automatically based on the host value in config.kdl. Use this for drivers (e.g., nvidia-utils) or hardware packages (e.g., tlp).

    modules/*.kdl: Loaded only if its name is in enabled_modules. Use this for software sets (e.g., gaming.kdl, development.kdl).

Module Management

You don't have to edit config.kdl manually.

To see all available modules (and their status):

```bash
declarch module list
```
To enable a new module:

```bash
declarch module enable gaming
```
To disable a module:

```bash
declarch module disable gaming
```
System Synchronization

declarch sync Your main command. This will:

    Read all KDL files.

    Compare against installed packages.

    Install any missing packages (like the gaming module you just enabled).

declarch sync --prune This command is powerful. It does the same as sync, but it will also uninstall any packages that are:

    In the state.json file (managed by declarch),

    BUT are no longer in your KDL files (like the gaming module you just disabled).

This is the recommended workflow to keep your system clean.

Advanced Features (Examples)

Excludes Use exclude in your host file to block a package from a module.
Code snippet
```bash
// ~/.config/declarch/hosts/laptop.kdl
// Don't install 'neofetch' from 'base', install 'fastfetch' instead
packages fastfetch
exclude neofetch
```
Conflicts (Safety) declarch will stop with an error if you try to enable two modules that conflict.
Code snippet
```bash
// ~/.config/declarch/modules/hyprland.kdl
description "Hyprland compositor"
packages hyprland
conflicts sway
```
Version Pinning (Warning) declarch will warn you if your installed version doesn't match. (Auto-downgrade is not currently supported).
Code snippet
```bash
// ~/.config/declarch/modules/base.kdl
// Will warn if git is not version 1.0.0
packages "git=1.0.0" vim
```
---

License

This project is licensed under the MIT License.
