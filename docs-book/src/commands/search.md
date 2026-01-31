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
# Search for a package (AUR only by default)
declarch search firefox

# Search in specific backends using --backends flag
declarch search bat --backends aur,npm

# Search in specific backends using prefix syntax
declarch search npm:prettier
declarch search cargo:ripgrep
declarch search yarn:prettier

# Limit results to 5 per backend
declarch search rust --limit 5

# Unlimited results
declarch search rust --limit all

# Show only installed matches
declarch search vim --installed-only

# Show only available (not installed) packages
declarch search python --available-only
```

## How It Works

The search command queries multiple package repositories in parallel and displays results grouped by backend. Installed packages are marked with a checkmark (✓).

### Supported Backends

| Backend | Search Status | Notes |
|---------|---------------|-------|
| **AUR** | ✅ Full Support | Via AUR RPC API, limited to 10 results by default |
| **npm** | ✅ Full Support | Via `npm search --json`, limited to 10 results by default |
| **yarn** | ✅ Full Support | Via npm registry API, limited to 10 results by default |
| **pnpm** | ✅ Full Support | Via npm registry API, limited to 10 results by default |
| **bun** | ✅ Full Support | Via npm registry API, limited to 10 results by default |
| **cargo** | ✅ Full Support | Via `cargo search`, limited to 10 results by default |
| **brew** | ✅ Full Support | Via `brew search` command, limited to 10 results by default |
| **Flatpak** | ✅ Full Support | Via `flatpak search` command, limited to 10 results by default |
| **Soar** | ✅ Full Support | Via `soar search --json`, limited to 10 results by default |
| **Pip** | ❌ Deprecated | `pip search` was deprecated in 2023, will not be implemented |
| **Custom** | ⚠️ Not Implemented | Needs backend.kdl syntax design (future enhancement) |

### Backend Settings Behavior

The `backend_mode` and `backends` settings affect search behavior:

- **`backend_mode: auto`** (default): Searches AUR only. Use `--backends` flag or `backend:query` syntax to search other backends.
- **`backend_mode: enabled-only`**: Only searches backends listed in the `backends` setting

**Example:**
```bash
# Default: AUR only
declarch search rust

# Search specific backends using flag
declarch search rust --backends aur,npm,cargo

# Search specific backends using prefix
declarch search npm:prettier
declarch search cargo:ripgrep

# Or use enabled-only mode
declarch settings set backend_mode "enabled-only"
declarch settings set backends "aur,npm"
declarch search rust  # Now searches AUR and npm
```

**Note:** The `--backends` flag and `backend:query` syntax always override settings (intentional design for flexibility).

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

### Search npm Only

```bash
$ declarch search prettier --backends npm

Searching Npm...
Found 10 package(s) matching 'prettier':

Npm:
  prettier 3.3.3 - Code formatter using eslint
  prettier-plugin-sql 1.4.0 - Format SQL files
  @prettier/plugin-php 0.22.2 - Prettier PHP plugin
  ...
```

### Search Brew Only

```bash
$ declarch search ffmpeg --backends brew

Searching Brew...
Found 10 package(s) matching 'ffmpeg':

Brew:
  ffmpeg 6.1.0 - Play/stream/record video and audio
  ffmpeg@2.8 2.8.22 - Play/stream/record video and audio
  ffmpeg@4 4.4.4 - Play/stream/record video and audio
  ffmpeg@5 5.1.2 - Play/stream/record video and audio
  libav 0.8.21 - Play/stream/record video and audio
  ...
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
# Default: 10 results per backend
$ declarch search rust

Searching Aur...
Found 42 packages matching 'rust' --limit 10 (showing 10):

Aur:
  rust 1.77.0 - Systems programming language
  rust-analyzer 2024-03-18 - Rust compiler
  rust-musl 1.77.0 - Musl-based Rust toolchain
  ...

# Custom limit
$ declarch search rust --limit 5

Searching Aur...
Found 42 packages matching 'rust' --limit 5 (showing 5):

Aur:
  rust 1.77.0 - Systems programming language
  rust-analyzer 2024-03-18 - Rust compiler
  ...

# Unlimited results (no limit message)
$ declarch search rust --limit all
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

### Pip Search Not Supported

Pip search will not be implemented since `pip search` was deprecated by PyPI in 2023.

### Custom Backend Search

User-defined custom backends (defined in `~/.config/declarch/backends.kdl`) **now support search**! You need to add a `search` block to your backend configuration.

#### Custom Backend Search Syntax

Add search configuration to your backend in `~/.config/declarch/backends.kdl`:

```kdl
backend "my-pm" {
    binary "my-pm"
    install "my-pm install {packages}"
    remove "my-pm remove {packages}"

    // Search with JSON API
    search "curl -s 'https://api.example.com/search?q={query}'" {
        format "json"
        json_path "results"
        name_key "name"
        version_key "version"
        desc_key "description"
    }
}
```

#### Supported Search Formats

**1. JSON Format** (for APIs returning JSON):
```kdl
search "curl -s 'https://api.example.com/search?q={query}'" {
    format "json"
    json_path "results"           // Path to results array
    name_key "name"               // Field name for package name
    version_key "version"         // Field name for version (optional)
    desc_key "description"        // Field name for description (optional)
}
```

**2. Tab-Separated Format**:
```kdl
search "my-pm search {query}" {
    format "tab"
    name_col 0    // Column index for package name
    desc_col 1    // Column index for description (optional)
}
```

**3. Whitespace-Separated Format**:
```kdl
search "my-pm search {query}" {
    format "whitespace"
    name_col 0
    desc_col 2
}
```

**4. Regex Format**:
```kdl
search "my-pm search {query}" {
    format "regex"
    regex "^(\\S+)\\s+-\\s+(.+)$"   // Pattern to match
    name_group 1                    // Capture group for name
    desc_group 2                    // Capture group for description
}
```

#### Error Handling

If a custom backend doesn't have search configured, you'll see:
```
⚠  Search from custom backend 'my-pm' is not working. Add 'search' configuration to ~/.config/declarch/backends.kdl
ℹ  See documentation for custom backend search syntax examples
```

### Default Search Scope

By default, `declarch search` only searches AUR. To search other backends, use:
- `--backends` flag: `declarch search rust --backends npm,cargo`
- `backend:query` syntax: `declarch search npm:prettier`

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
