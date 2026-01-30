# Installation

Install declarch on Arch Linux.

## From AUR (Recommended)

```bash
paru -S declarch
```

or

```bash
yay -S declarch
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

- Arch Linux or Arch-based distribution
- An AUR helper (`paru` or `yay`)
- Rust toolchain (for building from source)

## What's Next?

- [Quick Start](quick-start.md) - Get started in 3 minutes
- [Basic Concepts](basic-concepts.md) - Understand how declarch works
