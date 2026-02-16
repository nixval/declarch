# MCP Adapter v1 (Placeholder)

Status: design placeholder for external MCP adapter.

## Goal

Provide MCP tools that wrap stable `declarch` machine outputs.
Keep adapter external first (no in-process MCP runtime in declarch core).

Note:
- MCP protocol defines transport/procedure behavior.
- Client-side config file schema (where you put `command/args/env`) is client-specific.

## Reference binary (experimental)

This repository now includes an experimental adapter binary:

```bash
cargo run --bin declarch-mcp
```

Notes:
- Stdio JSON-RPC style (line-delimited JSON requests).
- Reads `DECLARCH_BIN` env var to locate declarch binary (default: `declarch`).
- Intended as reference baseline, not final production protocol.

## Suggested read-only tools

- `declarch_info`
- `declarch_list`
- `declarch_lint`
- `declarch_search`
- `declarch_sync_preview`

## Optional write tool (guarded)

- `declarch_sync_apply`
  - Disabled by default
  - Requires env: `DECLARCH_MCP_ALLOW_APPLY=1`
  - Requires tool arg: `confirm: "APPLY_SYNC"`

## Command mapping

- `declarch_info`
  - `declarch info --format json --output-version v1`
- `declarch_list`
  - `declarch info --list --format json --output-version v1`
- `declarch_lint`
  - `declarch lint --format json --output-version v1`
- `declarch_search`
  - `declarch search "<query>" --format json --output-version v1`
- `declarch_sync_preview`
  - `declarch --dry-run sync --format json --output-version v1`
- `declarch_sync_apply`
  - `declarch sync --yes`

## Safety model

- Start read-only only.
- Write/apply uses explicit confirmation token + environment gate.
- Keep stderr warnings visible in adapter logs.
