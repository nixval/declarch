# Policy, Hooks, and Editor Behavior (Advanced)

This page documents runtime behavior that often impacts safety and automation.

## 1) Policy block: what it controls

Example:

```kdl
policy {
    protected "linux" "systemd"
    orphans "ask"
    require_backend "true"
    forbid_hooks "false"
    on_duplicate "warn"
    on_conflict "warn"
}
```

Meaning (practical):
- `protected`: package names that should not be removed by prune-style flows.
- `orphans`: orphan handling strategy (`keep`, `remove`, `ask` depending on backend flow).
- `require_backend`: force explicit backend declaration; avoid implicit backend behavior.
- `forbid_hooks`: hard block hook execution even when CLI uses `--hooks`.
- `on_duplicate`: duplicate declaration policy (`warn` or `error`).
- `on_conflict`: cross-backend conflict policy (`warn` or `error`).

Related checks:

```bash
declarch lint --mode validate
declarch lint --mode duplicates
declarch lint --mode conflicts
```

## 2) Hooks execution policy (safety gate)

Hooks are intentionally gated by multiple conditions.

### Required conditions to execute hooks

1. Hooks exist in config.
2. Config explicitly enables hooks:

```kdl
experimental {
    "enable-hooks"
}
```

3. CLI call includes `--hooks`.
4. `policy.forbid_hooks` is not blocking hooks.

If one condition is missing, hooks are skipped/blocked.

### Hook phases

Supported lifecycle phases include:
- `pre-sync`, `post-sync`
- `on-success`, `on-failure`
- `pre-install`, `post-install`
- `pre-remove`, `post-remove`
- `on-update`

### Hook command safety rules

Hook command validation rejects risky patterns, including:
- embedded `sudo` in command string
- path traversal patterns like `../`
- unsafe characters outside allowed safe set

Runtime details:
- hook timeout is enforced
- behavior on failure depends on hook error policy (`warn`, `required`, `ignore`)

## 3) Editor selection policy (`declarch edit`)

Current runtime priority (actual behavior):

1. `editor` value from root config (`declarch.kdl`)
2. `$VISUAL`
3. `$EDITOR`
4. fallback to `nano`

If configured editor is missing in `PATH`, declarch warns and falls through to next option.

Useful edit flags:

```bash
declarch edit mymodule --preview --number
declarch edit mymodule --validate-only
declarch edit mymodule --auto-format
declarch edit mymodule --backup
```

## 4) Self-update ownership policy

`self-update` is intentionally constrained by install ownership.

If installation is owned by an external package manager (AUR/Homebrew/Scoop/Winget), declarch does not force direct overwrite and instead shows update hints for that package manager.

Use package-manager-native update command in those cases.

## 5) Recommended safe flow

```bash
declarch lint --mode validate
declarch lint --mode duplicates
declarch lint --mode conflicts
declarch --dry-run sync --hooks
declarch sync --hooks
```

If hooks are not expected to run, verify:
- `experimental { "enable-hooks" }`
- `--hooks` is present
- `policy.forbid_hooks` is not enabled
