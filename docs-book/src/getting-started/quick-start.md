# Quick Start

Get started with declarch in 3 minutes.

## Install

```bash
paru -S declarch
```

## Setup

```bash
# Initialize with a pre-made config
declarch init shell/dms

# Install your first packages
declarch install bat fzf ripgrep

# Done!
```

That's it! You now have:
- ✅ `bat` - Better cat
- ✅ `fzf` - Fuzzy finder
- ✅ `ripgrep` - Better grep

All managed declaratively in `~/.config/declarch/modules/others.kdl`.

## What Just Happened?

1. **`declarch init`** created your config directory and fetched a community config
2. **`declarch install`** added packages to `modules/others.kdl` and installed them
3. Your system is now in sync with your config

## Next Steps

```bash
# Add more packages
declarch install brave firefox

# Add to specific module
declarch install neovim --module dev

# Update system
declarch sync --update
```

## Need Help?

- [All Commands](../commands/) - Complete command reference
- [Examples](../examples/) - Real-world configurations
