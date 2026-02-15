# search

Search packages across configured backends.

## Usage

```bash
declarch search [OPTIONS] <QUERY>
```

## Examples

```bash
declarch search firefox
declarch search firefox -b aur
declarch search firefox -b aur,flatpak
declarch search npm:typescript

declarch search firefox --installed-only
declarch search firefox --local
declarch search firefox --limit all
```

## Options

| Option | Description |
|--------|-------------|
| `-b, --backends <NAMES>` | filter by backend(s) |
| `--limit <N\|all\|0>` | max per backend (default 10) |
| `--installed-only` | installed matches only |
| `--available-only` | available matches only |
| `--local` | search local installed package list |

## Notes

- Results stream backend by backend.
- Missing backend binaries are skipped with warning.
