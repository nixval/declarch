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
        platforms "linux"
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

## Required Fields

| Field | Description |
|-------|-------------|
| `binary` | Command to execute |
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

### tsv

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

## Placeholders

Commands can use placeholders:

| Placeholder | Expands to |
|-------------|------------|
| `{packages}` | Space-separated package names |
| `{binary}` | The binary command |

## Example: Apt Backend

```kdl
backend "apt" {
    meta {
        title "APT"
        description "Debian/Ubuntu package manager"
        platforms "linux"
    }
    
    binary "apt"
    
    list "apt list --installed" {
        format "regex"
        regex {
            pattern "^([^/]+)/[^ ]+ ([^ ]+)"
            name_group 1
            version_group 2
        }
    }
    
    install "apt install -y {packages}"
    remove "apt remove -y {packages}"
    
    needs_sudo "true"
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
declarch check validate
```

## See Also

- [KDL Syntax](../configuration/kdl-syntax.md)
- Backends in `~/.config/declarch/backends/` for examples
