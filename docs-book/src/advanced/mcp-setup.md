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

## Quick copy: Codex (`~/.codex/config.toml`)

```toml
[mcp_servers.declarch]
command = "/absolute/path/to/target/release/declarch-mcp"
args = []

[mcp_servers.declarch.env]
DECLARCH_BIN = "/absolute/path/to/target/release/declarch"
XDG_CONFIG_HOME = "/absolute/path/to/repo/.dev/config"
XDG_STATE_HOME = "/absolute/path/to/repo/.dev/state"
XDG_CACHE_HOME = "/absolute/path/to/repo/.dev/cache"
```

Optional guarded apply:

```toml
[mcp_servers.declarch.env]
DECLARCH_MCP_ALLOW_APPLY = "1"
```

## Quick copy: Gemini (`~/.gemini/antigravity/mcp_config.json`)

```json
{
  "mcpServers": {
    "declarch": {
      "command": "/absolute/path/to/target/release/declarch-mcp",
      "args": [],
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

## Quick copy: Qwen (`~/.qwen/settings.json`)

Based on your current schema, `mcpServers` entries support `command` + `args`.
If env fields are not supported by your current version, wrap with shell export:

```json
{
  "mcpServers": {
    "declarch": {
      "command": "bash",
      "args": [
        "-lc",
        "DECLARCH_BIN=/absolute/path/to/target/release/declarch XDG_CONFIG_HOME=/absolute/path/to/repo/.dev/config XDG_STATE_HOME=/absolute/path/to/repo/.dev/state XDG_CACHE_HOME=/absolute/path/to/repo/.dev/cache /absolute/path/to/target/release/declarch-mcp"
      ]
    }
  }
}
```

Enable guarded apply in Qwen wrapper command by adding:

```bash
DECLARCH_MCP_ALLOW_APPLY=1
```

inside the same `bash -lc` command string.

## Quick copy: Claude Code

Your current `~/.claude/settings.json` is focused on env + permissions and does not show a direct
`mcpServers` block in this file.

Use your Claude MCP server config location/schema, then map it to the same payload:
- command: `.../declarch-mcp`
- env: `DECLARCH_BIN`, `XDG_CONFIG_HOME`, `XDG_STATE_HOME`, `XDG_CACHE_HOME`
- optional env: `DECLARCH_MCP_ALLOW_APPLY=1` (only when you want write/apply)

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

- Different clients use different config file paths/field names.
- Keep the same command/env payload, then adapt to each client’s schema.
- Start with read-only tools first, then enable apply only when needed.
- Do not commit real API keys/tokens from client config files.
