# edit

Open config in your $EDITOR.

## Usage

```bash
declarch edit [TARGET]
```

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
