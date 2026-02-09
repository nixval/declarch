# search

Search for packages across backends.

## Usage

```bash
declarch search [OPTIONS] <QUERY>
```

## Examples

### Basic Search

```bash
declarch search firefox
```

Searches all backends.

### Search Specific Backend

```bash
# Single backend
declarch search firefox -b aur

# Multiple backends
declarch search firefox -b aur,flatpak
```

### Filter Results

```bash
# Only installed
declarch search firefox --installed-only

# Only available
declarch search firefox --available-only

# Limit results
declarch search firefox --limit 5
```

## Options

| Option | Description |
|--------|-------------|
| `-b, --backends <NAMES>` | Filter by backends |
| `--limit <N>` | Max results per backend |
| `--installed-only` | Show only installed |
| `--available-only` | Show only available |
