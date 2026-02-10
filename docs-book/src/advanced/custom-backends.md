# Custom Backends

Create backends for unsupported package managers.

## Backend Structure

A backend is a KDL file in `~/.config/declarch/backends/`:

```kdl
// backends/mybackend.kdl
backend "mybackend" {
    meta {
        title "My Backend"
        description "Custom package manager integration"
        maintained "your-name"
        platforms "linux" "macos"
    }
    
    binary "my-pm"
    
    list "my-pm list" {
        format "whitespace"
        name_col 0
        version_col 1
    }
    
    install "my-pm install {packages}"
    remove "my-pm remove {packages}"
    
    needs_sudo "true"
}
```

## Meta Block

The `meta` block provides metadata about your backend:

```kdl
meta {
    title "Backend Display Name"
    description "What this backend does"
    maintained "author-name"
    tags "tag1" "tag2" "tag3"
    url "https://docs-url.com"
    homepage "https://project-homepage.com"
    license "MIT"
    created "2024-01-01"
    platforms "linux" "macos" "windows"
    requires "dependency-name"
    install-guide "curl -fsSL https://install.sh | sh"
}
```

### Meta Fields

| Field | Required | Description | Example |
|-------|----------|-------------|---------|
| `title` | Yes | Display name of the backend | `"NPM"` |
| `description` | Yes | Short description | `"Node Package Manager"` |
| `maintained` | No | Maintainer name | `"nixval"` |
| `tags` | No | Searchable tags | `"nodejs" "javascript"` |
| `url` | No | Documentation URL | `"https://docs.npmjs.com"` |
| `homepage` | No | Project homepage | `"https://www.npmjs.com"` |
| `license` | No | License type | `"MIT"` |
| `created` | No | Creation date (YYYY-MM-DD) | `"2024-01-01"` |
| `platforms` | No | Supported platforms | `"linux" "macos" "windows"` |
| `requires` | No | Required dependencies | `"nodejs"` |
| `install-guide` | No | Installation instructions | `"curl ... \| sh"` |

**Note:** Fields with value `"-"` are hidden when displaying backend information.

## Required Fields

| Field | Description |
|-------|-------------|
| `binary` | Command to execute (can have multiple: `binary "cmd1" "cmd2"`) |
| `list` | Command to list installed packages |
| `install` | Command to install packages |
| `remove` | Command to remove packages |

## Output Formats

The `format` directive tells declarch how to parse output:

### whitespace

```
package-name version
```

```kdl
list "cmd -Q" {
    format "whitespace"
    name_col 0
    version_col 1
}
```

### tsv (Tab-Separated)

```
name\tversion
```

```kdl
list "cmd list" {
    format "tsv"
    name_col 0
    version_col 1
}
```

### json

For standard JSON output:

```json
[{"name": "pkg", "version": "1.0"}]
```

```kdl
list "cmd --json" {
    format "json"
    name_key "name"
    version_key "version"
}
```

For JSON with nested path:

```kdl
list "cmd --json" {
    format "json"
    json_path "dependencies"
    name_key "name"
    version_key "version"
}
```

### json_lines / jsonl / ndjson

For newline-delimited JSON (one JSON object per line):

```kdl
list "cmd --json" {
    format "json_lines"
    name_key "pkg_name"
    version_key "version"
}
```

This format automatically skips invalid JSON lines (like log messages).

### npm_json

For NPM-style pseudo-array format:

```
[
  {"name": "pkg1", "version": "1.0"}
  ,
  {"name": "pkg2", "version": "2.0"}
]
```

```kdl
list "npm list -g --json" {
    format "npm_json"
    name_key "name"
    version_key "version"
}
```

### regex

For custom parsing with regular expressions:

```kdl
list "cmd list" {
    format "regex"
    regex "^([\\w-]+)\\s+([\\d.]+)"
    name_group 1
    version_group 2
}
```

Or with a regex block for more complex patterns:

```kdl
search "cmd search {query}" {
    format "regex"
    regex {
        line "^(.+?)\\s+-\\s+(.+)$"
        name_group 1
        desc_group 2
    }
}
```

## Search Configuration

Add search capability to your backend:

```kdl
search "cmd search {query}" {
    format "whitespace"
    name_col 0
    desc_col 1
}
```

Search formats support the same options as `list`, plus:

```kdl
search "cmd search {query}" {
    format "json"
    name_key "name"
    version_key "version"
    desc_key "description"
}
```

## Additional Commands

### Update and Upgrade

```kdl
update "cmd update"
upgrade "cmd upgrade"
```

### Purge (remove with config)

```kdl
purge "cmd purge {packages}"
```

### Autoremove

```kdl
autoremove "cmd autoremove"
```

## Options

### noconfirm flag

For backends that support non-interactive mode:

```kdl
noconfirm "-y"
// or
noconfirm "--yes"
// or
noconfirm "--noconfirm"
```

### needs_sudo

For backends requiring root privileges:

```kdl
needs_sudo "true"
```

### fallback

Specify a fallback backend if this one is unavailable:

```kdl
fallback "pacman"
```

## Placeholders

Commands can use placeholders:

| Placeholder | Expands to |
|-------------|------------|
| `{packages}` | Space-separated package names |
| `{binary}` | The binary command |
| `{query}` | Search query string |

## Complete Example: Mise Backend

```kdl
// backends/mise.kdl
backend "mise" {
    meta {
        title "Mise"
        description "The front-end to your dev environment. Manage multiple language versions."
        maintained "nixval"
        tags "version-manager" "nodejs" "python" "rust" "golang"
        url "https://mise.jdx.dev"
        homepage "https://mise.jdx.dev"
        license "MIT"
        created "2024-01-01"
        platforms "linux" "macos"
        requires "none"
        install-guide "curl https://mise.run | sh"
    }
    
    binary "mise"
    sudo "false"
    
    // List installed tools
    list "mise list --installed" {
        format "regex"
        regex "^([\\w-]+)\\s+([\\d.]+)"
        name_group 1
        version_group 2
    }
    
    // Install tools
    install "mise use --global {packages}"
    
    // Remove tools
    remove "mise uninstall --global {packages}"
    
    // Update all tools
    update "mise upgrade"
    
    // Search available tools
    search "mise search {query}" {
        format "whitespace"
        name_col 0
        desc_col 1
    }
}
```

## Installing Your Backend

1. Create the file:
   ```bash
   mkdir -p ~/.config/declarch/backends
   # Create your-backend.kdl
   ```

2. Import it in `backends.kdl`:
   ```kdl
   imports {
       "backends/your-backend.kdl"
   }
   ```

3. Use it:
   ```kdl
   pkg {
       your-backend {
           package-name
       }
   }
   ```

## Testing

Test your backend:

```bash
# Validate the KDL syntax
declarch check

# Test list command
declarch list

# Test search (if configured)
declarch search your-backend:query
```

## Publishing to Registry

To share your backend with the community:

1. Fork `nixval/declarch-packages`
2. Add your backend to `backends/your-backend.kdl`
3. Submit a pull request

## See Also

- [KDL Syntax](../configuration/kdl-syntax.md)
- Backends in `~/.config/declarch/backends/` for examples
- [Official Backend Registry](https://github.com/nixval/declarch-packages/tree/main/backends)
