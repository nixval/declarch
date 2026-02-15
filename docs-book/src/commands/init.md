# init

Use this command to bootstrap config, add backends, or fetch remote config.

## Usage

```bash
declarch init [OPTIONS] [SOURCE]
```

## Most common usage

```bash
# first setup
declarch init

# add one backend
declarch init --backend npm

# add multiple
declarch init --backend pnpm,yarn
# or
declarch init --backend pnpm yarn

# discover remote backends
declarch init --list backends
```

## Files created by first init

```text
~/.config/declarch/
├── declarch.kdl
├── backends.kdl
├── backends/
└── modules/
    └── base.kdl
```

`state.json` is not in config dir. It lives in your OS state directory.
Linux example:

```text
~/.local/state/declarch/state.json
```

Tip:

```bash
declarch info --doctor
```

to print actual config/state paths for your machine.

## Useful options

| Option | Description |
|--------|-------------|
| `--backend <NAME>...` | adopt backend definition(s) |
| `--list <WHAT>` | list `backends` or `modules` |
| `--host <NAME>` | set hostname template |
| `--restore-backends` | recreate `backends.kdl` |
| `--restore-declarch` | recreate `declarch.kdl` |
| `--force` | overwrite existing files where supported |

## Remote source example

```bash
declarch init username/dotfiles
declarch init username/dotfiles:minimal
```
