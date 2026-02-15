# Integration Examples (API, MCP, Plugins)

This page is practical brainstorming translated into concrete examples.
Goal: show what can be built on top of declarch without making core too complex.

## Plugin examples (external executables)

### 1) Security audit plugin

`declarch-ext-security-audit`

- Reads package inventory from:
  - `declarch info --list --format json --output-version v1`
- Checks CVE databases (or internal advisory feed).
- Outputs risk summary for CI.

### 2) Notification plugin

`declarch-ext-notify`

- Runs after sync in CI/local automation.
- Sends concise report to Discord/Slack/Telegram.
- Useful for shared infra/team setups.

### 3) Team policy plugin

`declarch-ext-policy-team`

- Rejects forbidden packages/backends before apply.
- Enforces naming/module conventions.
- Great for organization-wide baseline rules.

### 4) Export plugin

`declarch-ext-export`

- Converts declarch managed state/plan into other formats:
  - CSV/JSON inventory
- infra report artifacts
- internal dashboard feed

Protocol reference:
- `docs/contracts/v1/extensions-protocol.md`

## MCP examples (recommended first)

Read-only MCP tools are low-risk and high value.

Candidate MCP tools:
- `declarch_info`
- `declarch_lint`
- `declarch_search`
- `declarch_sync_preview`

All can shell out to `declarch` and parse machine output (`v1` envelope).

## API examples (optional later)

If eventually needed, API can mirror existing command surfaces:

- `GET /info`
- `GET /lint`
- `GET /search?q=...`
- `POST /sync/preview`

Important: keep API responses aligned with the same `v1` envelope contract.

## Integrations in CI/CD

- PR validation:
  - `declarch lint --strict`
  - `declarch sync preview`
- Artifact export:
  - store `info/list` machine output as CI artifacts.
- Team notifications:
  - send drift warnings or preview summaries.

## Suggested rollout

1. Stabilize machine output contracts (in progress).
2. Build read-only MCP adapter externally.
3. Add extension discovery/runtime (`declarch ext`) incrementally.
4. Re-evaluate embedded API only when demanded by real usage.
