# Installation

Pick one method.

## Arch Linux (AUR)

```bash
paru -S declarch
# or
yay -S declarch
```

## Linux (install script)

```bash
curl -sSL https://raw.githubusercontent.com/nixval/declarch/main/install.sh | sh
```

## macOS (install script)

```bash
curl -sSL https://raw.githubusercontent.com/nixval/declarch/main/install.sh | sh
```

The installer picks the correct macOS target (`x86_64` or `aarch64`).
Current status: **alpha preview**.
After install, it runs lightweight smoke checks (`--help`, `info`).

## Windows (PowerShell, alpha preview)

```powershell
irm https://raw.githubusercontent.com/nixval/declarch/main/install.ps1 | iex
```

Current status: **alpha preview**.
After install, it runs lightweight smoke checks (`--help`, `info`).

## Manual binary install

```bash
wget https://github.com/nixval/declarch/releases/latest/download/declarch-x86_64-unknown-linux-gnu.tar.gz
tar xzf declarch-x86_64-unknown-linux-gnu.tar.gz
sudo install declarch /usr/local/bin/
```

## Build from source

```bash
cargo install declarch --git https://github.com/nixval/declarch
```

## Verify

```bash
declarch --version
declarch --help
```

## Updating

- Package manager install (AUR/Homebrew/etc): update with your package manager.
- Script/manual install (`curl`/`wget`): update with `declarch self-update`.

If you are on macOS/Windows, run:

```bash
declarch info --doctor
declarch --dry-run sync
```

Next: [Quick Start](./quick-start.md)
