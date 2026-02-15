# MCP Setup (Technical)

This page shows how to connect `declarch` with MCP clients while keeping core behavior agnostic and safe.

## Scope

- MCP adapter is external (`declarch-mcp`), not in-process plugin code.
- Core `declarch` logic is unchanged.
- Read-only tools are available by default.
- Write/apply action is guarded.
- `declarch-mcp` is a local stdio adapter.
- declarch does not ship a built-in public HTTP MCP server in this guide.

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

Important:
- If you only use local stdio, `command` (and optional `args`/`env`) is enough.
- `url`/`headers` are only for clients that connect to remote HTTP MCP servers.

## Common fields across clients (quick map)

- Local stdio (process on your machine):
  - usually `command`
  - optional `args`, `env`
  - some clients also accept/require `type: "stdio"` or `type: "local"`
- Remote MCP over network:
  - usually `url` (or `httpUrl` / `serverUrl` depending client)
  - optional/required `type` (`http`, `streamableHttp`, `sse`)
  - optional `headers` for auth

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

## Binary requirement

Your MCP client must be able to run `declarch-mcp`.

- If installed in PATH: use `command = "declarch-mcp"` (or JSON equivalent).
- If not in PATH: set `command` to the full binary path.

`DECLARCH_BIN` is optional and only needed when you want to force a specific `declarch` binary.

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

If `declarch-mcp` is not in PATH yet, set `command` to the full binary path.

## Quick copy: Codex (`~/.codex/config.toml`)

```toml
[mcp_servers.declarch]
command = "declarch-mcp"
args = []

```

Optional if you want to force a specific declarch binary:

```toml
[mcp_servers.declarch.env]
DECLARCH_BIN = "/path/to/declarch"
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
        "DECLARCH_BIN": "/path/to/declarch"
      }
    }
  }
}
```

## Quick copy: Qwen (`~/.qwen/settings.json`)

Some Qwen setups accept `command` + `args` directly.
If env fields are not supported in your version, use a shell wrapper:

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

Claude setup can vary by app version and integration mode.
If your config file does not expose a direct `mcpServers` block, use Claude's MCP config location
for your version and map the same payload below.

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
- When a client supports both local and remote MCP, start with local stdio first.

## Optional: custom XDG (advanced)

Custom XDG is optional and usually only needed for isolated test setups.
