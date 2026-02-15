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

The installer will pick the correct macOS target (`x86_64` or `aarch64`).
Status: **experimental (alpha)**.

## Windows (PowerShell, experimental alpha)

```powershell
irm https://raw.githubusercontent.com/nixval/declarch/main/install.ps1 | iex
```

Status: **experimental (alpha)**.

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

Next: [Quick Start](./quick-start.md)
