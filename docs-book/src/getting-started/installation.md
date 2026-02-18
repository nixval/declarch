# Installation

Pick one method.

## Arch Linux (AUR)

```bash
paru -S declarch-bin
# or
yay -S declarch-bin
```

## Linux/macOS (install script)

```bash
curl -sSL https://raw.githubusercontent.com/nixval/declarch/main/install.sh | sh
```

After install, it runs lightweight smoke checks (`--help`, `info`).

Do this for each of distro for easy to githubusercontent
```bash
decl init --backend aur paru yay pacman // for Arch distro base
decl init --backend nala apt // for Debian/ubuntu distro base
decl init --backend dnf5 // for Red Hat distro base
decl init --backend zypper // for SUSE distro base
decl init --backend brew // for macOS

// or you can custom it based on package manager preference
// I already manage it at nixval/package-manager
// or see the list at `decl init --list backend`
```

## Windows (PowerShell, alpha preview)

```powershell
irm https://raw.githubusercontent.com/nixval/declarch/main/install.ps1 | iex
```

Current status: **alpha preview**.
After install, it runs lightweight smoke checks (`--help`, `info`).
Still dont have the configuration, but possibly can configured to have winget, choco, scoop, etc.

## Manual binary install

```bash
wget https://github.com/nixval/declarch/releases/latest/download/declarch-x86_64-unknown-linux-gnu.tar.gz
tar xzf declarch-x86_64-unknown-linux-gnu.tar.gz
sudo install declarch /usr/local/bin/
```

## Build from source

```bash
cargo install declarch --git https://github.com/nixval/declarch
```

Source builds can be slower on first run because Cargo compiles the full dependency graph.
If you want a faster setup path, prefer prebuilt release binaries via install script or package manager.

## Verify

```bash
declarch --version
declarch --help
declarch info --doctor -v
```

Next: [Quick Start](./quick-start.md)
