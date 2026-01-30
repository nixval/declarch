# Installation

Install declarch on your system.

## From AUR (Arch Linux)

**Recommended method for Arch Linux users:**

```bash
paru -S declarch
```

or

```bash
yay -S declarch
```

## Binary Release (Any Linux)

**Quick install (one-liner):**

```bash
curl -sSL https://raw.githubusercontent.com/nixval/declarch/v0.5.1/install.sh | sh
```

This script automatically detects your architecture (x86_64 or aarch64) and installs declarch.

**Manual download:**

```bash
# Download (x86_64)
wget https://github.com/nixval/declarch/releases/download/v0.5.1/declarch-x86_64-unknown-linux-gnu.tar.gz
tar xzf declarch-x86_64-unknown-linux-gnu.tar.gz
sudo install declarch /usr/local/bin/

# Or for ARM64 (aarch64)
wget https://github.com/nixval/declarch/releases/download/v0.5.1/declarch-aarch64-unknown-linux-gnu.tar.gz
tar xzf declarch-aarch64-unknown-linux-gnu.tar.gz
sudo install declarch /usr/local/bin/
```

## From Source

```bash
# Clone repository
git clone https://github.com/nixval/declarch.git
cd declarch

# Build and install
cargo build --release
sudo install target/release/declarch /usr/local/bin/
```

## Verify Installation

```bash
declarch --version
```

Output:
```
declarch 0.5.1
```

## Requirements

- **Arch Linux**: An AUR helper (`paru` or `yay`)
- **Other Linux**: curl or wget for downloading binaries
- **From Source**: Rust toolchain (1.92.0 or later)

## What's Next?

- [Quick Start](quick-start.md) - Get started in 3 minutes
- [Basic Concepts](basic-concepts.md) - Understand how declarch works
