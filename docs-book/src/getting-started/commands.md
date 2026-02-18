# Commands

This page maps current CLI behavior (`declarch --help`) into one practical guide.

## Declarative-first mindset

Declarch is designed to be config-first:

```kdl
pkg {
    aur { bat fzf ripgrep }
    npm { typescript }
}
```

Then apply:

```bash
declarch --dry-run sync
declarch sync
```

You can still use `install` for speed, but your source of truth stays in KDL modules.

## Global flags (all commands)

- `-v, --verbose`
- `-q, --quiet`
- `-y, --yes`
- `-f, --force`
- `--dry-run`
- `--format table|json|yaml`
- `--output-version v1` (for machine output contracts)

## `init`

Usage:

```bash
declarch init [OPTIONS] [SOURCE]
```

Main usage:

```bash
declarch init
declarch init --backend npm
declarch init --backend apt,nala
declarch init --list backends
declarch init --list modules
```

`SOURCE` supports:
- `user/repo`
- `user/repo:variant`
- `user/repo/branch`
- `gitlab.com/user/repo`
- `https://...kdl`
- registry module name (example: `desktop/hyprland`)

Important options:
- `--backend <NAME>...`
- `--list <backends|modules>`
- `--local` (create local module, skip registry lookup)
- `--host <NAME>`
- `--restore-declarch`

## `install`

Usage:

```bash
declarch install [OPTIONS] <PACKAGES>...
```

Examples:

```bash
declarch install aur:bat aur:fzf aur:ripgrep
declarch install npm:typescript
declarch install bat fzf ripgrep --backend aur
declarch install firefox --module browsers --backend aur
```

Options:
- `-b, --backend <BACKEND>`
- `-m, --module <MODULE>`
- `--no-sync`

## `sync`

Usage:

```bash
declarch sync [OPTIONS] [SUBCOMMAND]
```

Common flow:

```bash
declarch --dry-run sync
declarch sync
```

Core options (default sync + `update` + `prune`):
- `--target <TARGET>`
- `--profile <NAME>`
- `--host <NAME>`
- `--diff`
- `--noconfirm`
- `--hooks`
- `--modules <MODULES>`

Subcommands:

```bash
declarch sync update
declarch sync prune
declarch sync cache --backend npm
declarch sync upgrade --backend npm --no-sync
```

Subcommand-specific options:
- `sync cache`: `-b, --backend <BACKEND>...`
- `sync upgrade`: `-b, --backend <BACKEND>...`, `--no-sync`

Hook behavior and gating details are documented in:
[Policy, Hooks, and Editor Behavior](../advanced/policy-hooks-editor.md).

## `search`

Usage:

```bash
declarch search [OPTIONS] <QUERY>
```

Examples:

```bash
declarch search firefox
declarch search firefox -b aur,flatpak
declarch search npm:typescript
declarch search firefox --installed-only
declarch search firefox --available-only
declarch search firefox --local
declarch search firefox --limit all
```

Options:
- `-b, --backends <BACKENDS>`
- `--limit <NUM|all|0>`
- `--installed-only`
- `--available-only`
- `--local`

## `info`

Usage:

```bash
declarch info [OPTIONS] [QUERY]
```

Modes/examples:

```bash
declarch info
declarch info --doctor
declarch info --plan
declarch info --list
declarch info --list --scope all
declarch info --list --scope orphans
declarch info --list --scope synced
declarch info --list --scope unmanaged
declarch info aur:bat
```

Options:
- `--doctor`
- `--plan`
- `--list`
- `--scope all|orphans|synced|unmanaged`
- `--backend <BACKEND>`
- `--package <PACKAGE>`
- `--profile <NAME>`
- `--host <NAME>`
- `--modules <MODULES>`

## `lint`

Usage:

```bash
declarch lint [OPTIONS]
```

Examples:

```bash
declarch lint
declarch lint --mode validate
declarch lint --mode duplicates
declarch lint --mode conflicts --backend aur
declarch lint --diff
declarch lint --fix
declarch lint --strict
declarch lint --benchmark
```

State-related options:

```bash
declarch lint --repair-state
declarch lint --state-rm soar:firefox
declarch lint --state-rm package --state-rm-backend soar
declarch lint --state-rm-backend soar --state-rm-all
```

Main options:
- `--mode all|validate|duplicates|conflicts`
- `--backend <BACKEND>`
- `--diff`, `--fix`, `--strict`, `--benchmark`
- `--repair-state`
- `--state-rm <IDS>`
- `--state-rm-backend <BACKEND>`
- `--state-rm-all`
- `--profile <NAME>`
- `--host <NAME>`
- `--modules <MODULES>`

## `edit`

Usage:

```bash
declarch edit [OPTIONS] [TARGET]
```

Examples:

```bash
declarch edit
declarch edit mydotfiles --create
declarch edit mydotfiles --preview --number
declarch edit mydotfiles --validate-only
declarch edit mydotfiles --backup
```

Options:
- `-p, --preview`
- `--number` (requires `--preview`)
- `-c, --create`
- `--auto-format`
- `--validate-only`
- `-b, --backup`

Editor resolution policy details:
[Policy, Hooks, and Editor Behavior](../advanced/policy-hooks-editor.md).

## `switch`

Usage:

```bash
declarch switch [OPTIONS] <OLD_PACKAGE> <NEW_PACKAGE>
```

Examples:

```bash
declarch switch neovim neovim-git
declarch switch firefox aur:firefox-nightly --backend aur
declarch switch firefox firefox-nightly --dry-run
```

Options:
- `--backend <BACKEND>`

## Hidden/internal commands

Not shown in main help, but available for advanced/internal workflows:
- `declarch self-update` (script/manual install update path)
- `declarch completions <shell>`
- `declarch ext`
