# MCP Setup (Technical)

This page shows how to connect `declarch` with MCP clients while keeping core behavior agnostic and safe.

## Scope

- MCP adapter is external (`declarch-mcp`), not in-process plugin code.
- Core `declarch` logic is unchanged.
- Read-only tools are available by default.
- Write/apply action is guarded.

## Important: MCP config format is client-specific

MCP protocol standardizes message transport (JSON-RPC over stdio/http), not one universal
`mcpServers` file schema for every app.

So:
- `command + args + env` pattern is common.
- exact file path and key names depend on each client.
- your client config may look different and still be valid.

## Build binaries

From repo root:

```bash
cargo build --release
```

Expected binaries:

- `target/release/declarch`
- `target/release/declarch-mcp`

## Recommended standard environment

Use your real declarch paths (normal user setup):

```bash
declarch init
```

Linux defaults are typically:
- config: `~/.config/declarch`
- state: `~/.local/state/declarch`

Use this to confirm your actual paths:

```bash
declarch info --doctor
```

## Optional isolated environment (dev/testing only)

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

Use this as conceptual template (adapt keys to your client schema):

```json
{
  "mcpServers": {
    "declarch": {
      "command": "/absolute/path/to/target/release/declarch-mcp"
    }
  }
}
```

That is enough for standard usage.
`declarch` will use normal OS default paths automatically.

## Quick copy: Codex (`~/.codex/config.toml`)

```toml
[mcp_servers.declarch]
command = "/absolute/path/to/target/release/declarch-mcp"
args = []

[mcp_servers.declarch.env]
DECLARCH_BIN = "/absolute/path/to/target/release/declarch"
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
        "DECLARCH_BIN": "/absolute/path/to/target/release/declarch"
      }
    }
  }
}
```

## Quick copy: Qwen (`~/.qwen/settings.json`)

Based on your current schema, `mcpServers` entries support `command` + `args`.
If env fields are not supported by your current version, minimal wrapper:

```json
{
  "mcpServers": {
    "declarch": {
      "command": "bash",
      "args": [
        "-lc",
        "DECLARCH_BIN=/absolute/path/to/target/release/declarch /absolute/path/to/target/release/declarch-mcp"
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

## Optional: custom XDG (advanced)

Custom XDG is optional and mainly useful for isolated test environments.

Example:

```bash
XDG_CONFIG_HOME="$PWD/.dev/config" \
XDG_STATE_HOME="$PWD/.dev/state" \
XDG_CACHE_HOME="$PWD/.dev/cache" \
declarch-mcp
```
