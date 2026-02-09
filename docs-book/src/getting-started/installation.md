# Installation

## Arch Linux (Recommended)

```bash
paru -S declarch
# or
yay -S declarch
```

## Any Linux (Binary)

```bash
curl -sSL https://raw.githubusercontent.com/nixval/declarch/main/install.sh | sh
```

This downloads and installs the latest release to `/usr/local/bin/`.

## Manual Install

```bash
# Download latest release
wget https://github.com/nixval/declarch/releases/latest/download/declarch-x86_64-unknown-linux-gnu.tar.gz

# Extract
tar xzf declarch-x86_64-unknown-linux-gnu.tar.gz

# Install
sudo install declarch /usr/local/bin/
```

## From Source

```bash
cargo install declarch --git https://github.com/nixval/declarch
```

## Verify Installation

```bash
declarch --version
declarch --help
```

## Next

→ [Quick Start Guide](./quick-start.md)
