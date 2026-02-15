# MCP Setup (Technical)

This page shows how to connect `declarch` with MCP clients while keeping core behavior agnostic and safe.

## Scope

- MCP adapter is external (`declarch-mcp`), not in-process plugin code.
- Core `declarch` logic is unchanged.
- Read-only tools are available by default.
- Write/apply action is guarded.

## Build binaries

From repo root:

```bash
cargo build --release
```

Expected binaries:

- `target/release/declarch`
- `target/release/declarch-mcp`

## Recommended isolated environment

```bash
mkdir -p .dev/config .dev/state .dev/cache
XDG_CONFIG_HOME="$PWD/.dev/config" \
XDG_STATE_HOME="$PWD/.dev/state" \
XDG_CACHE_HOME="$PWD/.dev/cache" \
./target/release/declarch init
```

## MCP adapter quick test (raw stdio)

List tools:

```bash
echo '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}' \
| DECLARCH_BIN="$PWD/target/release/declarch" ./target/release/declarch-mcp
```

Preview sync:

```bash
echo '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"declarch_sync_preview","arguments":{}}}' \
| DECLARCH_BIN="$PWD/target/release/declarch" ./target/release/declarch-mcp
```

## Tools exposed

- `declarch_info`
- `declarch_list`
- `declarch_lint`
- `declarch_search`
- `declarch_sync_preview`
- `declarch_sync_apply` (guarded)

## Quick copy: generic MCP stdio config

Use this as template for any MCP client that supports stdio server config:

```json
{
  "mcpServers": {
    "declarch": {
      "command": "/absolute/path/to/target/release/declarch-mcp",
      "env": {
        "DECLARCH_BIN": "/absolute/path/to/target/release/declarch",
        "XDG_CONFIG_HOME": "/absolute/path/to/repo/.dev/config",
        "XDG_STATE_HOME": "/absolute/path/to/repo/.dev/state",
        "XDG_CACHE_HOME": "/absolute/path/to/repo/.dev/cache"
      }
    }
  }
}
```

## Quick copy: enable guarded apply

If you explicitly want AI to run `declarch sync` apply:

```json
{
  "mcpServers": {
    "declarch": {
      "command": "/absolute/path/to/target/release/declarch-mcp",
      "env": {
        "DECLARCH_BIN": "/absolute/path/to/target/release/declarch",
        "DECLARCH_MCP_ALLOW_APPLY": "1"
      }
    }
  }
}
```

And MCP call must include:

```json
{
  "name": "declarch_sync_apply",
  "arguments": {
    "confirm": "APPLY_SYNC"
  }
}
```

Without both guards, apply is rejected.

## Client notes

- Claude Code, Codex, OpenCode, and similar clients use different config file paths/field names.
- Keep the same command/env payload, then adapt to each client’s schema.
- Start with read-only tools first, then enable apply only when needed.
