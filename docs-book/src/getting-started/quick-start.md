# Quick Start


## 1) Initialize

```bash
declarch init
```

Expected structure:

```text
~/.config/declarch/
├── declarch.kdl
├── backends/
└── modules/
    └── base.kdl
```

Use this anytime to see actual config/state paths on your OS:

```bash
declarch info --doctor -v
```

## 2) Adopt backend(s) first

```bash
`````bash
decl init --backend aur paru yay pacman // for Arch distro base
decl init --backend nala apt // for Debian/ubuntu distro base
decl init --backend dnf5 // for Red Hat distro base
decl init --backend zypper // for SUSE distro base
decl init --backend brew // for macOS

// or you can custom it based on package manager preference
// I already manage it at nixval/package-manager
// or see the list at `decl init --list backend`
// then adopt it `decl init pnpm npm bun soar flatpak`, etc
````

## 3) Add packages

### Declaratively config

add packages inside pkg { <backendname> { <yourpackages> } }
// ~/.config/declarch/modules/mydotfiles.kdl
meta {
    title "My dotfiles"
}
pkg {
    aur { hyprland waybar }
    flatpak { firefox }
    soar { gimp }
    npm {
        @opencode-ai/sdk
        oh-my-opencode
    }
}

```bash
decl edit // will edit declarch.kdl
decl edit mydotfiles --create // will create new file and editted directly
decl edit mydotfiles // will edit ./modules/mydotfiles.kdl

// then just do `decl sync`
decl sync --dry-run // if not sure yet
decl lint --fix // if something error syntax happen
decl sync prune // if there some packages that deleted from configuration
decl sync // install what you type in config
```

### Direct install
```bash
declarch install aur:bat aur:fzf aur:ripgrep
declarch install npm:typescript
declarch install bat fzf ripgrep --backend aur
// this will automatically written in ./modules/others.kdl
```

## 4) Preview and apply

```bash
declarch --dry-run sync
declarch sync
```

## 5) Add more backends when needed

```bash
declarch init --backend npm
declarch init --backend pnpm,yarn
# also valid
declarch init --backend pnpm yarn
```

That is the core workflow.
