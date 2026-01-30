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
curl -L https://github.com/nixval/declarch/releases/download/v0.5.1/declarch-x86_64-unknown-linux-gnu.tar.gz | tar xz && sudo install declarch /usr/local/bin/
```

**Manual download:**

```bash
# Download
wget https://github.com/nixval/declarch/releases/download/v0.5.1/declarch-x86_64-unknown-linux-gnu.tar.gz

# Extract
tar xzf declarch-x86_64-unknown-linux-gnu.tar.gz

# Install
sudo install declarch /usr/local/bin/
```

**For ARM64 (aarch64):**

```bash
curl -L https://github.com/nixval/declarch/releases/download/v0.5.1/declarch-aarch64-unknown-linux-gnu.tar.gz | tar xz && sudo install declarch /usr/local/bin/
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
