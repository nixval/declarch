# MCP Personal Testing Guide (declarch)

This guide is for your own local testing with MCP clients (Claude Code, Codex, OpenCode, etc).

## 1) Build binaries

From repo root:

```bash
cargo build --release
```

You should have:

- `./target/release/declarch`
- `./target/release/declarch-mcp`

## 2) Prepare isolated test env (recommended)

```bash
mkdir -p .dev/config .dev/state .dev/cache
XDG_CONFIG_HOME="$PWD/.dev/config" \
XDG_STATE_HOME="$PWD/.dev/state" \
XDG_CACHE_HOME="$PWD/.dev/cache" \
./target/release/declarch init
```

## 3) Manual MCP sanity test (without client)

List tools:

```bash
echo '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}' \
| DECLARCH_BIN="$PWD/target/release/declarch" ./target/release/declarch-mcp
```

Call read-only tool:

```bash
echo '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"declarch_info","arguments":{}}}' \
| DECLARCH_BIN="$PWD/target/release/declarch" ./target/release/declarch-mcp
```

Search example:

```bash
echo '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"declarch_search","arguments":{"query":"firefox","limit":5}}}' \
| DECLARCH_BIN="$PWD/target/release/declarch" ./target/release/declarch-mcp
```

## 4) Optional apply (AI can run installation)

`declarch_sync_apply` is guarded.

Required:
- env `DECLARCH_MCP_ALLOW_APPLY=1`
- argument `confirm: "APPLY_SYNC"`

Example:

```bash
echo '{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"declarch_sync_apply","arguments":{"confirm":"APPLY_SYNC"}}}' \
| DECLARCH_BIN="$PWD/target/release/declarch" DECLARCH_MCP_ALLOW_APPLY=1 ./target/release/declarch-mcp
```

If you do not set both guard conditions, apply will be rejected.

## 5) Generic MCP client config (stdio)

Use this as template and adapt to your client’s config format:

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

To allow apply from AI, add:

```json
"DECLARCH_MCP_ALLOW_APPLY": "1"
```

Only enable that when you really want write/apply actions.

## 6) Suggested first prompts in MCP client

- "List available declarch MCP tools."
- "Run declarch_info and summarize current status."
- "Run declarch_sync_preview and explain what will change."
- (Optional, guarded) "Run declarch_sync_apply with confirmation token."

## 7) Current scope

Working tools:
- `declarch_info`
- `declarch_list`
- `declarch_lint`
- `declarch_search`
- `declarch_sync_preview`
- `declarch_sync_apply` (guarded)

Notes:
- Machine output contract used: `v1`
- Core declarch remains agnostic; MCP adapter is external binary.
