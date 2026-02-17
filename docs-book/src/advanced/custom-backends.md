# Custom Backends (Advanced)

This page documents custom backend authoring for `backends/*.kdl` under your declarch config directory.

## File placement and import

1. Create file (Linux example):

```bash
mkdir -p ~/.config/declarch/backends
$EDITOR ~/.config/declarch/backends/mypm.kdl
```

Use `declarch info --doctor` to print your exact config path on current OS.

2. Import in `declarch.kdl`:

```kdl
backends {
    "backends/mypm.kdl"
}
```

## Minimal valid backend

```kdl
backend "mypm" {
    binary "mypm"

    list "{binary} list" {
        format "whitespace"
        name_col 0
        version_col 1
    }

    install "{binary} install {packages}"
    remove "{binary} remove {packages}"
}
```

## Required and optional fields

### Required

- `backend "name" { ... }`
- `binary "..."` (single or multiple)
- `install "...{packages}..."`

### Strongly recommended

- `list "..." { ... }` for state/introspection
- `remove "...{packages}..."`
- `search "...{query}..." { ... }`

### Optional commands

- `search_local "...{query}..." { ... }`
- `update "..."`
- `upgrade "..."`
- `cache_clean "..."`
- `noconfirm "-y"`
- `needs_sudo true`
- `fallback "other-backend"`
- `env KEY="VALUE"`

`"-"` can be used on some commands to explicitly disable capability.

## Optional meta block (recommended)

```kdl
meta {
    title "My Backend"
    description "What this backend does"
    kdl-maintainer "your-name"
    tags "linux" "package-manager"
    homepage "https://example.com"
    platforms "linux"
    requires "mybinary"
    install-guide "https://example.com/install"
}
```

Compatibility:
- Canonical keys: `kdl-maintainer`, `install-guide`
- Legacy aliases accepted: `maintainer`, `maintained`, `installation_guide`

## Placeholders

- `{binary}`: resolved executable (supports multi-binary and fallback scenarios)
- `{packages}`: space-separated package arguments
- `{query}`: search query text

If `binary` has multiple options, include `{binary}` in command templates.

## Output format parsers

Supported `format` values:

- `whitespace`
- `tsv`
- `json`
- `json_lines` / `jsonl` / `ndjson`
- `npm_json`
- `json_object_keys`
- `regex`

### Whitespace example

```kdl
list "{binary} -Q" {
    format "whitespace"
    name_col 0
    version_col 1
}
```

### JSON example (nested path)

```kdl
list "{binary} list --json" {
    format "json"
    json {
        path "dependencies"
        name_key "name"
        version_key "version"
    }
}
```

Compatibility note: flat keys (`json_path`, `name_key`, `version_key`) are also accepted.

### Search JSON example

```kdl
search "{binary} search {query} --json" {
    format "json"
    json {
        path "results"
        name_key "name"
        version_key "version"
        desc_key "description"
    }
}
```

### Regex example

```kdl
search "{binary} search {query}" {
    format "regex"
    regex "^([^\s]+)\s+-\s+(.+)$"
    name_group 1
    desc_group 2
}
```

## Validation expectations

Backend validation enforces:

- `install` must include `{packages}`
- `remove` (if set) must include `{packages}`
- `search`/`search_local` (if set) should include `{query}`
- parser-specific required keys must exist (e.g. `name_key` for JSON list)

## Fallback example

```kdl
backend "nala" {
    binary "nala"
    fallback "apt"

    list "{binary} list --installed" {
        format "regex"
        regex "^(\S+)/"
        name_group 1
    }

    install "{binary} install -y {packages}"
    remove "{binary} remove -y {packages}"
    needs_sudo true
}
```

## Testing checklist

```bash
# parse + config checks
declarch lint --mode validate

# backend visibility
declarch info

# optional search smoke test
declarch search mypm:foo --limit 5
```

Then run a limited dry-run sync:

```bash
declarch --dry-run sync --target mypm
```

## Publishing

If backend works across environments, contribute to:

- https://github.com/nixval/declarch-packages
