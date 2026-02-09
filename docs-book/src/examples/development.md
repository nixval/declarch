# Development Setup

A complete development environment.

## Prerequisites

```bash
# Add required backends
declarch init --backend npm
```

## Structure

```
modules/
├── base.kdl      # Essential tools
├── dev.kdl       # Development tools
└── langs.kdl     # Programming languages
```

## modules/dev.kdl

```kdl
pkg {
    aur {
        neovim
        tmux
        docker
        docker-compose
    }
    
    pacman {
        git
        github-cli
        jq
    }
}
```

## modules/langs.kdl

```kdl
pkg {
    aur {
        rustup
    }
    
    npm {
        typescript
        ts-node
        prettier
        eslint
    }
}
```

## declarch.kdl

```kdl
imports {
    "modules/base.kdl"
    "modules/dev.kdl"
    "modules/langs.kdl"
}
```

## Apply

```bash
declarch sync
```

## Post-Install

Some tools need extra setup:

```bash
# Rust
rustup default stable

# Docker
sudo systemctl enable docker
sudo usermod -aG docker $USER
```
