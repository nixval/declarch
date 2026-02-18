# Command Overview

If you only memorize a few commands, memorize these.

## Core loop

```bash
declarch init
declarch install <backend:package...>
declarch sync
```

## Daily commands

- `declarch --dry-run sync` - safe preview mode.
- `declarch search <query>` - find package candidates.
- `declarch info` - status, doctor, list, and package reasoning.
- `declarch lint` - config quality checks.

## Helpful sync variants

- `declarch sync update` - refresh indexes + sync.
- `declarch sync prune` - remove unmanaged packages.
- `declarch sync cache` - clean cache(s).
- `declarch sync upgrade` - run backend upgrades.
- `declarch sync --profile <name>` - opt-in profile layer.
- `declarch sync --host <name>` - opt-in host layer.

## Global flags

- `-y, --yes`
- `-f, --force`
- `-v, --verbose`
- `-q, --quiet`
- `--dry-run`
- `--output-version v1` (machine-readable envelope, with `--format json|yaml`)

## Beginner advice

Use preview often and add backends gradually.
When using `install`, always specify backend via `backend:pkg` or `--backend`.
Keep base config portable; use `--profile`/`--host` only for extra machine-specific packages.
If a backend is not for your current OS, declarch can skip it and continue.


# init

Use this command to bootstrap config, add backends, or fetch remote config.

## Usage

```bash
declarch init [OPTIONS] [SOURCE]
```

## Most common usage

```bash
# first setup
declarch init

# add one backend
declarch init --backend npm

# add multiple
declarch init --backend pnpm,yarn
# or
declarch init --backend pnpm yarn

# discover remote backends
declarch init --list backends
```
```bash
# discover remote modules
declarch init --list modules

# packages to install virtual-manager
declarch init linux/vm

# packages to install completed hyprland
declarch init desktop/hyprland

# packages to install common gui apps
declarch init apps/flatpak-common

declarch init --dry-run // to make sure before surely install it
```


## Files created by first init

```text
~/.config/declarch/
├── declarch.kdl
├── backends/
└── modules/
    └── base.kdl
```

`state.json` is not in config dir. It lives in your OS state directory.
Linux example:

```text
~/.local/state/declarch/state.json
```

Tip:

```bash
declarch info --doctor
```

This prints actual config/state paths for your machine.

## Useful options

| Option | Description |
|--------|-------------|
| `--backend <NAME>...` | adopt backend definition(s) |
| `--list <WHAT>` | list `backends` or `modules` |
| `--host <NAME>` | set hostname template |
| `--restore-declarch` | recreate `declarch.kdl` |
| `--force` | overwrite existing files where supported |

## Remote source example

```bash
declarch init username/dotfiles
declarch init username/dotfiles:minimal
```

so if you have dotfiles that want user to install dependencies, you can provide it in your root repo with file `declarch.kdl` or `declarch-withsuffix.kdl`
```bash
decl init yourgithubaccount/yourreponame
decl init yourgitlabaccount/yourreponame:withsuffix
```


# install

Add packages to config quickly.

## Usage

```bash
declarch install [OPTIONS] <PACKAGES>...
```

## Important

`install` requires explicit backend now.

Use one of these styles:
- `backend:package` per package
- `--backend <name>` for all packages

## Common examples

```bash
declarch install aur:neovim
declarch install aur:bat aur:fzf aur:ripgrep

declarch install npm:typescript

declarch install org.mozilla.firefox --backend flatpak
declarch install firefox --module browsers --backend aur
```

## What happens

1. Package entries are written to a module (`modules/others.kdl` by default).
2. `declarch sync` runs automatically, unless `--no-sync` is used.
3. If sync is cancelled or fails, config changes are rolled back.

## Options

| Option | Description |
|--------|-------------|
| `-b, --backend <NAME>` | force backend for all packages |
| `-m, --module <NAME>` | target module file |
| `--no-sync` | edit only, skip sync |


# sync

Synchronize your system with config.

## Usage

```bash
declarch sync [COMMAND] [OPTIONS]
```

## Commands

| Command | Description |
|---------|-------------|
| `(default)` | normal sync |
| `update` | refresh indexes then sync |
| `prune` | remove unmanaged packages |
| `cache` | clean backend cache |
| `upgrade` | run backend upgrades |

## Typical flow

```bash
declarch --dry-run sync
declarch sync
```

## More examples

```bash
declarch sync update
declarch sync prune
declarch sync --target firefox
declarch sync --hooks
declarch sync --profile desktop
declarch sync --host vps-1
```

## Common options

| Option | Description |
|--------|-------------|
| `--target <NAME>` | sync one package/scope |
| `--profile <NAME>` | activate `profile "NAME" { ... }` block |
| `--host <NAME>` | activate `host "NAME" { ... }` block |
| `--noconfirm` | skip backend prompt flags |
| `--hooks` | enable lifecycle hooks |
| `--modules <NAME>...` | temporary extra modules |
| `--diff` | show plan diff |

## Machine output (v1)

```bash
declarch --dry-run sync --format json --output-version v1
declarch --dry-run sync --format yaml --output-version v1
```

This emits one machine envelope summary for integrations.

## Hook safety gate

Even with `--hooks`, hooks are blocked unless you explicitly opt in from config:

```kdl
experimental {
    "enable-hooks"
}
```

Without that block, `declarch` shows hook entries but does not execute them.

## Cross-machine behavior

If you share one config across Linux/macOS/Windows in the future, some backends may not fit every OS.
Declarch will skip incompatible backends and continue the sync flow.
The same skip behavior is used by related flows like `sync cache` and `sync upgrade`.


# search

Search packages across configured backends.

## Usage

```bash
declarch search [OPTIONS] <QUERY>
```

## Examples

```bash
declarch search firefox
declarch search firefox -b aur
declarch search firefox -b aur,flatpak
declarch search npm:typescript

declarch search firefox --installed-only
declarch search firefox --local
declarch search firefox --limit all
```

## Options

| Option | Description |
|--------|-------------|
| `-b, --backends <NAMES>` | filter by backend(s) |
| `--limit <N\|all\|0>` | max per backend (default 10) |
| `--installed-only` | installed matches only |
| `--available-only` | available matches only |
| `--local` | search local installed package list |

## Notes

- Results stream backend by backend.
- Missing backend binaries are skipped with warning.

## Machine output (v1)

```bash
declarch search firefox --format json --output-version v1
declarch search firefox --format yaml --output-version v1
```

In this mode, results are emitted as one machine envelope instead of streaming display.


# info

Show status, diagnosis, and package reasoning in one place.

## Usage

```bash
declarch info [QUERY] [FLAGS]
```

## Common examples

```bash
# status (default)
declarch info

# doctor
declarch info --doctor

# list views
declarch info --list
declarch info --list --scope orphans
declarch info --list --scope synced
declarch info --list --scope unmanaged

# reasoning (replaces old explain)
declarch info bat
declarch info aur:bat
declarch info system/base

declarch info --plan
```

## Useful flags

- `--doctor`: run diagnostics
- `--plan`: show sync install/remove drift reasoning
- `--list`: list managed packages
- `--scope orphans`: with `--list`, show orphan packages only
- `--scope synced`: with `--list`, show synced packages only
- `--scope unmanaged`: with `--list`, show installed packages outside declarch config adoption
- `--backend <name>`: filter status/list output by backend
- `--package <name>`: filter status output by package name
- `--profile`, `--host`, `--modules`: apply optional context for reasoning mode

## Machine output (v1)

For integrations/scripts, you can request contract envelope output:

```bash
declarch info --format json --output-version v1
declarch info --list --format yaml --output-version v1
```

Use this for scripts, CI, and integrations that need stable structured output.

## Notes

- Use one mode per call: status, query, `--plan`, `--doctor`, or list mode.
- If a backend is not meant for current OS, checks can skip it gracefully.


# lint

Lint checks configuration quality before sync.

## Usage

```bash
declarch lint [FLAGS]
```

## Common examples

```bash
# full checks
declarch lint

# syntax/import checks only (replaces old check validate)
declarch lint --mode validate

# focused checks
declarch lint --mode duplicates
declarch lint --mode conflicts --backend aur

# optional extras
declarch lint --diff
declarch lint --fix
declarch lint --strict
declarch lint --benchmark
declarch lint --repair-state

# state cleanup (state only, no uninstall)
declarch lint --state-rm soar:firefox --dry-run
declarch lint --state-rm soar:firefox
declarch lint --state-rm package --state-rm-backend soar
declarch lint --state-rm-backend soar --state-rm-all
```

## Flags

- `--mode all|validate|duplicates|conflicts`: lint scope
- `--backend <name>`: backend filter for package-level checks
- `--diff`: show planned install/remove drift
- `--fix`: apply safe automatic fixes
- `--strict`: warnings become blocking errors
- `--benchmark`: show elapsed time
- `--repair-state`: sanitize broken state entries (no manual JSON edits)
- `--state-rm <ids>`: remove state entries by `backend:package` or plain package name
- `--state-rm-backend <name>`: backend scope for plain names, or for backend-wide cleanup
- `--state-rm-all`: remove all tracked entries for `--state-rm-backend`
- `--profile`, `--host`, `--modules`: include optional overlays/modules

## Minimal playbook (recommended)

```bash
declarch lint --state-rm backend:package --dry-run
declarch lint --state-rm backend:package
declarch --dry-run sync
```

For more debugging flows: [Advanced Debugging](../advanced/debugging.md).

## Machine output (v1)

```bash
declarch lint --format json --output-version v1
declarch lint --format yaml --output-version v1
```

When this mode is used, lint prints structured envelope output for automation/integrations.

## Recommended flow

```bash
declarch lint
declarch lint --fix
declarch lint --strict
```


# edit

Open config in your editor.

## Usage

```bash
declarch edit [TARGET]
```

## Set editor

Put this in `declarch.kdl`:

```kdl
editor "nvim"
```

Editor priority:
1. `declarch.kdl` (`editor`)
2. `$EDITOR`
3. fallback `nano`

## Examples

```bash
# open main config
declarch edit

# open module
declarch edit base

# open backend config
declarch edit backends
```

After editing, run:

```bash
declarch --dry-run sync
```


# switch

Replace one package entry with another.

## Usage

```bash
declarch switch [OPTIONS] <OLD_PACKAGE> <NEW_PACKAGE>
```

## Examples

```bash
declarch switch neovim neovim-git
declarch switch firefox aur:firefox-nightly

# preview
declarch switch firefox firefox-nightly --dry-run
```

## What it does

1. removes old package from config
2. adds new package to config
3. syncs the change

## Options

| Option | Description |
|--------|-------------|
| `-b, --backend <NAME>` | backend scope |
| `--dry-run` | preview only |

## Cross-machine note

If the selected backend is not for your current OS, declarch skips switch safely.
You can keep that backend in the same shared config for other machines.


