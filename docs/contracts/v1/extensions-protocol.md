# Extension Protocol v1 (Placeholder)

Status: draft contract for external extensions.

Execution/runtime is not fully implemented yet in core.
Current `declarch ext` only discovers extension binaries in `PATH`.

## Discovery

Extensions are discovered by executable name prefix:

- `declarch-ext-*`

Examples:
- `declarch-ext-security-audit`
- `declarch-ext-notify`
- `declarch-ext-policy-team`

## Planned invocation model

Core idea:
1. `declarch` executes extension binary.
2. `declarch` sends JSON request on stdin.
3. Extension writes JSON response to stdout.
4. Non-zero exit code means operation failure.

## Request envelope (draft)

```json
{
  "version": "v1",
  "command": "run",
  "extension": "declarch-ext-security-audit",
  "context": {
    "cwd": "/path/to/repo",
    "os": "linux"
  },
  "input": {}
}
```

## Response envelope (draft)

```json
{
  "version": "v1",
  "ok": true,
  "data": {},
  "warnings": [],
  "errors": []
}
```

## Safety notes

- Extensions run outside declarch process boundary.
- Keep extension execution opt-in.
- Keep write operations explicit and confirmable.
