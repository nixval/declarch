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

## Why some examples are only `command`

For local stdio MCP, many clients can start a server process with just:

- `command`: executable name/path

So this is valid when `declarch-mcp` is already in PATH:

```json
{
  "mcpServers": {
    "declarch": {
      "command": "declarch-mcp"
    }
  }
}
```

Add other fields only when needed:

- `args`: if your server needs startup arguments
- `env`: if you need custom environment variables
- `type`, `url`, `headers`: for remote HTTP/SSE servers

## Common fields across clients (quick map)

- Local stdio (process on your machine):
  - usually `command`
  - optional `args`, `env`
  - some clients also accept/require `type: "stdio"` or `type: "local"`
- Remote MCP over network:
  - usually `url` (or `httpUrl` / `serverUrl` depending client)
  - optional/required `type` (`http`, `streamableHttp`, `sse`)
  - optional `headers` for auth

`npx` is also normal, not special:

```json
{
  "mcpServers": {
    "example": {
      "command": "npx",
      "args": ["-y", "some-mcp-server@latest"]
    }
  }
}
```

## Build binaries

From repo root:

```bash
cargo build --release
```

Expected binaries:

- `target/release/declarch`
- `target/release/declarch-mcp`

## What does `/absolute/path/to/...` mean?

That text is only a placeholder.
Replace it with your real file path.

Example from this repo:

- `/home/nixval/github/repo/nixval/tools/declarch/target/release/declarch-mcp`

How to get the real path quickly:

```bash
realpath target/release/declarch-mcp
realpath target/release/declarch
```

If you installed declarch system-wide, you can usually use command names directly:

```bash
which declarch
which declarch-mcp
```

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
| ./target/release/declarch-mcp
```

Preview sync:

```bash
echo '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"declarch_sync_preview","arguments":{}}}' \
| ./target/release/declarch-mcp
```

`DECLARCH_BIN` is optional.
Use it only when you want MCP adapter to call a specific declarch binary path.

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
      "command": "declarch-mcp"
    }
  }
}
```

That is enough for standard usage.
`declarch` will use normal OS default paths automatically.

If `declarch-mcp` is not in PATH yet, use real absolute path from `realpath`.

## Quick copy: Codex (`~/.codex/config.toml`)

```toml
[mcp_servers.declarch]
command = "declarch-mcp"
args = []

```

Optional if you want to force a specific declarch binary:

```toml
[mcp_servers.declarch.env]
DECLARCH_BIN = "/absolute/path/to/declarch"
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
      "command": "declarch-mcp",
      "args": []
    }
  }
}
```

Optional:

```json
{
  "mcpServers": {
    "declarch": {
      "command": "declarch-mcp",
      "env": {
        "DECLARCH_BIN": "/absolute/path/to/declarch"
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
        "declarch-mcp"
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
- optional env: `DECLARCH_BIN` (only when needed)
- optional env: `DECLARCH_MCP_ALLOW_APPLY=1` (only when you want write/apply)

## Quick copy: enable guarded apply

If you explicitly want AI to run `declarch sync` apply:

```json
{
  "mcpServers": {
    "declarch": {
      "command": "declarch-mcp",
      "env": {
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
