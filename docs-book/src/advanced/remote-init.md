# Remote Init (Advanced)

This page documents source resolution behavior for `declarch init [SOURCE]`.

## Supported source forms

```text
user/repo
user/repo:variant
user/repo/branch
gitlab.com/user/repo
https://example.com/path/declarch.kdl
registry/module
```

## Resolution behavior

### GitHub shorthand

```bash
declarch init username/dotfiles
```

Resolves to repository default branch and fetches `declarch.kdl`.

### Variant

```bash
declarch init username/dotfiles:minimal
```

Targets variant config (e.g. `declarch-minimal.kdl`).

### Branch

```bash
declarch init username/dotfiles/develop
```

Fetches from explicit branch path.

### GitLab

```bash
declarch init gitlab.com/username/dotfiles
```

### Direct URL

```bash
declarch init https://example.com/config.kdl
```

### Registry module

```bash
declarch init hyprland/niri-nico
```

## Operational flow

1. Resolve source candidates.
2. Download candidate content.
3. Validate KDL parseability.
4. Write to local config path.
5. Initialize missing local structure if needed.

## Safety recommendations

```bash
# inspect before applying changes
declarch init username/repo --dry-run
```

- treat remote config as untrusted input,
- review hooks and backend commands before full sync,
- prefer pinning branch/tag for reproducibility.

## Failure modes

- `not found`: wrong source path/variant/branch.
- `parse error`: remote file is not valid KDL.
- `network error`: transport or host availability issue.

## Debug workflow

```bash
declarch -v init username/repo
declarch check validate
```
