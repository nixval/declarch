# declarch Machine Contract v1 (Placeholder)

This folder defines machine-readable output contracts for integrations.

Status: placeholder contract for future API/MCP/plugin ecosystem.

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

## Notes

- Human/table output remains unchanged.
- v1 envelope is for machine consumers.
- Initial surfaces:
  - `info`
  - `lint`
  - `search` (planned output mode)
  - `sync preview` (planned output mode)

## Examples

- `info.json`
- `lint.json`
- `search.json`
- `sync-preview.json`
