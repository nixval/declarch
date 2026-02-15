# MCP Adapter v1 (Placeholder)

Status: design placeholder for external MCP adapter.

## Goal

Provide MCP tools that wrap stable `declarch` machine outputs.
Keep adapter external first (no in-process MCP runtime in declarch core).

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
  - `declarch sync preview --format json --output-version v1`

## Safety model

- Start read-only only.
- Add write/apply tools later with explicit confirmation token flow.
- Keep stderr warnings visible in adapter logs.
