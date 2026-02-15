# Integration Roadmap (API, MCP, Plugins)

Status: evolving roadmap.

Goal: keep `declarch` simple, while making integrations possible without rewriting core behavior.

## Principles

- Keep core CLI behavior beginner-friendly.
- Keep machine contracts explicit and versioned.
- Prefer external integrations over in-process plugin loading.
- Read-only integrations first, write operations later.

## Phase plan

### Phase 1: Contracts first (done)

- Define stable machine-output envelope (`v1`).
- Add examples for `info`, `lint`, `search`, and `sync preview`.
- Add CLI contract flag: `--output-version v1`.

### Phase 2: Extension protocol foundation

- Reserve extension model with executable discovery:
  - `declarch-ext-*`
- Add extension command surface:
  - `declarch ext`
- Keep this safe and incremental.

### Phase 3: MCP adapter (external)

- Build sidecar that calls `declarch` read-only commands.
- Initial tools:
  - `info`
  - `lint`
  - `search`
  - `sync preview`

### Phase 4: API mode (optional)

- Consider local API daemon only if real demand appears.
- API should reuse the same `v1` envelope contract.

## Why this path

- Minimal risk to current codebase.
- No forced deep architecture rewrite now.
- Future integrations can evolve around a stable output contract.

## Contract docs

See:
- `docs/contracts/v1/README.md`
- `docs/contracts/v1/mcp-adapter.md`

## Current implementation snapshot

- Added global contract flag: `--output-version v1`
- Added hidden extension command: `declarch ext`
- `declarch ext` now discovers `declarch-ext-*` binaries from `PATH`
- `v1` envelope implemented for:
  - `info` JSON/YAML
  - `info --list` JSON/YAML
  - `lint` JSON/YAML
  - `search` JSON/YAML
