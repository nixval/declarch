# search

Search for packages across configured backends.

## Usage

```bash
declarch search [OPTIONS] <QUERY>
```

## Examples

```bash
# Search all configured backends
declarch search firefox

# Single backend
declarch search firefox -b aur

# Multiple backends
declarch search firefox -b aur,flatpak

# Alternative syntax
declarch search npm:typescript

# Installed-only view
declarch search firefox --installed-only

# Local search mode (installed packages only)
declarch search firefox --local

# Unlimited results
declarch search firefox --limit all
```

## Options

| Option | Description |
|--------|-------------|
| `-b, --backends <NAMES>` | Filter by backend(s) |
| `--limit <N\|all\|0>` | Max results per backend (default 10) |
| `--installed-only` | Show only installed matches |
| `--available-only` | Show only available matches |
| `--local` | Use local installed-package search only |

## Notes

- Search runs backend-by-backend and streams results.
- If a backend binary is missing, that backend is skipped with warning.
