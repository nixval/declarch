# Remote Init

Fetch configs from remote repositories.

## GitHub Repos

### Basic

```bash
declarch init username/dotfiles
```

Fetches: `https://github.com/username/dotfiles/blob/main/declarch.kdl`

### With Variant

```bash
declarch init username/dotfiles:minimal
```

Fetches: `declarch-minimal.kdl`

### With Branch

```bash
declarch init username/dotfiles/develop
```

Fetches from `develop` branch.

## GitLab

```bash
declarch init gitlab.com/username/dotfiles
```

## Direct URL

```bash
declarch init https://example.com/config.kdl
```

## Registry

Official configs from the declarch registry:

```bash
declarch init hyprland/niri-nico
```

## How It Works

1. Downloads the config file
2. Saves to `~/.config/declarch/declarch.kdl`
3. Creates local modules directory
4. Runs `declarch sync`

## Security

Remote configs are downloaded to a temp location first. Review before applying:

```bash
# Preview only
declarch init username/repo --dry-run
```

## Private Repos

For private repos, set up SSH keys first:

```bash
# Test access
git ls-remote git@github.com:username/private-repo.git

# Then init
declarch init username/private-repo
```

## Troubleshooting

**"Config not found"**

- Check the repo has `declarch.kdl` in the root
- For variants, check `declarch-<variant>.kdl` exists

**"Network error"**

- Check internet connection
- Check GitHub/GitLab status
