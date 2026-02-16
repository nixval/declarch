# edit

Open config in your editor.

## Usage

```bash
declarch edit [TARGET]
```

## Set editor

Put this in `declarch.kdl`:

```kdl
editor "nvim"
```

Editor priority:
1. `declarch.kdl` (`editor`)
2. `$EDITOR`
3. fallback `nano`

## Examples

```bash
# open main config
declarch edit

# open module
declarch edit base

# open backend config
declarch edit backends
```

After editing, run:

```bash
declarch --dry-run sync
```
