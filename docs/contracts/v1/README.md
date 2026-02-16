# declarch Machine Contract v1 (Placeholder)

This folder defines machine-readable output contracts for integrations.

Status: staged rollout for future API/MCP/plugin ecosystem.

## Envelope

All v1 outputs SHOULD follow this shape:

```json
{
  "version": "v1",
  "command": "info",
  "ok": true,
  "data": {},
  "warnings": [],
  "errors": [],
  "meta": {
    "generated_at": "2026-02-15T00:00:00Z"
  }
}
```

## Rollout status

- Implemented now:
  - `declarch info --format json --output-version v1`
  - `declarch info --list --format json --output-version v1`
  - `declarch lint --format json --output-version v1`
  - `declarch search <query> --format json --output-version v1`
  - `declarch --dry-run sync --format json --output-version v1`
  - YAML also supported by replacing `json` with `yaml`.
- Human/table output remains unchanged.
- For now, using `--output-version v1` on unsupported commands returns a clear error.

## Examples

- `info.json`
- `lint.json`
- `search.json`
- `sync-preview.json`

## Related

- `extensions-protocol.md` (external plugin protocol placeholder)
- `mcp-adapter.md` (external MCP adapter mapping placeholder)
