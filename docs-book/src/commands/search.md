# declarch search

Search for packages across multiple backends.

## Usage

```bash
declarch search [OPTIONS] <QUERY>
```

## Options

- `<QUERY>` - Search query string (supports `backend:query` syntax)
- `-b, --backends <BACKENDS>` - Filter by backends (comma-separated)
- `--limit <NUM>` - Limit results per backend (default: 10, use `all` or `0` for unlimited)
- `--installed-only` - Show only installed packages
- `--available-only` - Show only available packages (not installed)

## Quick Start

```bash
# Search for a package in all backends
declarch search firefox

# Search in specific backends
declarch search bat --backends aur,npm

# Limit results to 5 per backend
declarch search rust --limit 5

# Unlimited results
declarch search rust --limit all

# Show only installed matches
declarch search vim --installed-only

# Show only available (not installed) packages
declarch search python --available-only

# Backend-specific search using alternative syntax
declarch search npm:prettier
```

## How It Works

The search command queries multiple package repositories in parallel and displays results grouped by backend. Installed packages are marked with a checkmark (✓).

### Supported Backends

| Backend | Search Status | Notes |
|---------|---------------|-------|
| **AUR** | ✅ Full Support | Via AUR RPC API, limited to 10 results by default |
| **Flatpak** | ⚠️ Not Implemented | Coming soon (needs `flatpak search` parsing) |
| **npm** | ⚠️ Not Implemented | Coming soon (needs `npm search --json` parsing) |
| **yarn/pnpm/bun** | ⚠️ Not Implemented | Coming soon (via npm registry API) |
| **Cargo** | ⚠️ Not Implemented | Coming soon (needs `cargo search` parsing) |
| **Pip** | ⚠️ Not Implemented | Coming soon (needs PyPI API) |
| **Brew** | ⚠️ Not Implemented | Coming soon (needs `brew search` parsing) |
| **Soar** | ⚠️ Not Implemented | Coming soon (needs Soar search support) |
| **Custom** | ⚠️ Not Implemented | Needs backend.kdl syntax design |

### Backend Settings Behavior

The `backend_mode` and `backends` settings affect search behavior:

- **`backend_mode: auto`** (default): Searches all backends that have search implemented (currently AUR only)
- **`backend_mode: enabled-only`**: Only searches backends listed in the `backends` setting

**Example:**
```bash
# Set to only search specific backends
declarch settings set backend_mode "enabled-only"
declarch settings set backends "aur"

# Now search will only look in AUR
declarch search rust
```

**Note:** The `--backends` flag always overrides settings (intentional design for flexibility).

### Distro-Aware Defaults

Default backends are automatically configured based on your Linux distribution:

- **Arch-based systems** (Arch, Manjaro, EndeavourOS): `aur,flatpak,soar,npm,cargo,pip,brew`
- **Non-Arch systems** (Debian, Fedora, etc.): `flatpak,soar,npm,cargo,pip,brew` (AUR excluded)

This is automatically detected when you run `declarch init` for the first time.

## Examples

### Basic Search

```bash
$ declarch search firefox

Searching Aur...
Found 15 package(s) matching 'firefox':

Aur:
  firefox 124.0-1 - Standalone web browser from mozilla.org
  firefox-beta 125.0-1 - Beta channel of the standalone web browser
  firefox-developer-hg 124.0-1 - Developer channel of the web browser
  ...
```

### Filter by Backend

```bash
$ declarch search bat --backends aur

Searching Aur...
Found 3 package(s) matching 'bat':

Aur:
  bat 0.24.0 - A cat(1) clone with wings
  bat-extras 2023.11.28 - Shell scripts that integrate bat with various tools
  bat-monokai-pro 0.5.0 - Monokai Pro theme for bat
```

### Installed Packages Only

```bash
$ declarch search ripgrep --installed-only

Found 2 package(s) matching 'ripgrep':

Aur:
  ripgrep ✓ 14.1.0 - A fast search tool
```

The checkmark (✓) indicates the package is already installed.

### Multiple Backends

```bash
$ declarch search prettier --backends aur,npm

Searching Aur...
Searching Npm...
Found 12 package(s) matching 'prettier':

Aur:
  prettier 3.2.5-1 - Prettier is an opinionated code formatter
  nodejs-prettier ✓ 3.2.5-1 - Prettier is an opinionated code formatter

Npm:
  prettier 3.3.3 - Code formatter using eslint
  prettier-plugin-sql 1.4.0 - Format SQL files
  ...
```

### Backend-Specific Query Syntax

You can use `backend:query` syntax as an alternative to `--backends` flag:

```bash
# These are equivalent:
declarch search npm:prettier
declarch search prettier --backends npm

# Search for "bat" in AUR:
declarch search aur:bat

# Search for "ripgrep" in Cargo:
declarch search cargo:ripgrep
```

**Note:** The `backend:query` syntax overrides the `--backends` flag if both are provided.

### Result Limiting

Control how many results are displayed per backend:

```bash
# Default: 10 results
declarch search firefox

# Custom limit
declarch search firefox --limit 5

# Unlimited results
declarch search firefox --limit all
declarch search firefox --limit 0
```

## Tips

1. **Use specific queries:**
   ```bash
   # Good - specific
   declarch search "rust analyzer"

   # Less specific - more results
   declarch search rust
   ```

2. **Combine with backend filtering:**
   ```bash
   # Search for CLI tools in AUR only
   declarch search ls --backends aur

   # Or use the shorthand syntax
   declarch search aur:ls
   ```

3. **Check if package is available:**
   ```bash
   # See what's available but not installed
   declarch search "neovim" --available-only
   ```

4. **Limit results for common names:**
   ```bash
   # "python" matches hundreds of packages - limit to 10
   declarch search python --limit 10
   ```

5. **Use settings to control default behavior:**
   ```bash
   # Only search specific backends by default
   declarch settings set backend_mode "enabled-only"
   declarch settings set backends "aur"
   ```

## Performance Notes

- Search queries are parallelized across backends
- AUR search uses the AUR RPC API (very fast)
- Default limit of 10 results per backend prevents overwhelming output
- Cache doesn't affect search - always fetches fresh results
- Network connection required for search to work
- Use `--limit 0` or `--limit all` for unlimited results (may be slow)

## Current Limitations

### Only AUR Search is Implemented

Currently, only AUR has working search functionality. Other backends (Flatpak, npm, Cargo, etc.) will show "Backend X does not support search" when queried.

This is a temporary limitation. Future releases will add search support for:
- Flatpak (via `flatpak search --json`)
- npm/yarn/pnpm/bun (via npm registry API)
- Cargo (via `cargo search` or crates.io API)
- Pip (via PyPI API)
- Brew (via `brew search`)

### Backend Settings Interaction

The `backend_mode` and `backends` settings affect:
- ✅ `sync` - Respects which backends to sync
- ✅ `search` - Respects which backends to search
- ❌ `install` - Does NOT check settings (can install to any backend)

This means you can install packages from any backend using `declarch install`, regardless of settings.

## Related

- [`install`](install.md) - Add found packages to your configuration
- [`info`](info.md) - Check installed package details
- [`sync`](sync.md) - Install packages from configuration
- [`settings`](settings.md) - Configure backend behavior
