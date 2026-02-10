# edit

Open config in your $EDITOR.

## Usage

```bash
declarch edit [TARGET]
```

## Configuration

Set your preferred editor in `declarch.kdl`:

```kdl
// Use vim (default if not specified)
editor "vim"

// Or use neovim
editor "nvim"

// Or use VS Code
editor "code"
```

**Priority order:**
1. `declarch.kdl` editor setting
2. `$EDITOR` environment variable
3. Fallback to `nano`

## Examples

### Edit Main Config

```bash
declarch edit
```

Opens `~/.config/declarch/declarch.kdl`

### Edit Module

```bash
declarch edit base
```

Opens `~/.config/declarch/modules/base.kdl`

### Edit Backends

```bash
declarch edit backends
```

Opens `~/.config/declarch/backends.kdl`

## After Editing

Remember to sync:

```bash
declarch sync
```
