# Development Environment Setup

Configuration optimized for software development.

## Configuration

```kdl
// ~/.config/declarch/declarch.kdl

meta {
    description "Development environment"
    host "dev-machine"
    tags "development" "programming"
}

// System packages
packages {
    neovim
    tmux
    lazygit
    gh
    postgresql
    redis
    docker
    kubectl
    terraform
    curl
    jq
}

// === DEVELOPMENT TOOLS ===

// Rust crates
packages:cargo {
    ripgrep      // Fast grep alternative
    fd-find      // Fast find alternative
    zoxide       // Smart directory navigation
    bat          // Enhanced cat with syntax highlighting
    tealdeer     // Simplified man pages
    xh           // Modern HTTP client
    mdbook       # Book creation from markdown
    cargo-edit   // Cargo extension for managing dependencies
}

// Node.js packages
packages:npm {
    # JavaScript/TypeScript
    typescript
    prettier
    eslint
    vite         # Fast build tool
    create-vite  # Project scaffolding

    # AWS SDK
    @aws-sdk/client-s3
}

// Python packages
packages:python {
    black        # Code formatter
    ruff         # Linter and formatter
    mypy         # Type checker
    jupyter      # Interactive notebooks
    poetry       # Dependency management
}

// Optional hooks
on-sync "notify-send 'Dev' 'Development tools updated'"
```

## What This Includes

### Core Development Tools (System)
- **Neovim** - Text editor
- **tmux** - Terminal multiplexer
- **lazygit** - Git TUI
- **gh** - GitHub CLI
- **Docker** - Container runtime
- **kubectl** - Kubernetes CLI
- **Terraform** - Infrastructure as Code

### Rust Development (via cargo)
- **ripgrep** - Fast text search
- **fd** - Fast file search
- **zoxide** - Smart directory navigation
- **bat** - Enhanced cat with syntax highlighting
- **tealdeer** - Simplified man pages
- **xh** - Modern HTTP client
- **mdbook** - Create books from markdown

### JavaScript/TypeScript (via npm)
- **TypeScript** - Type-safe JavaScript
- **Prettier** - Code formatter
- **ESLint** - Linter
- **Vite** - Fast build tool
- **create-vite** - Project scaffolding
- **AWS SDK** - Amazon S3 client

### Python (via pip)
- **black** - Code formatter
- **ruff** - Fast linter and formatter
- **mypy** - Static type checker
- **Jupyter** - Interactive notebooks
- **poetry** - Dependency management

## Development Workflow

```bash
# Initial setup
declarch init
declarch sync

# Start new project
mkdir my-project
cd my-project

# JavaScript/TypeScript project
npm init vite

# Search code
rg "function"     # ripgrep search
fd "test.rs"      # fd find files
zoxide proj       # zoxide jump to project

# HTTP testing
xh GET https://api.example.com

# Read docs
tldr tar          # simplified man pages
```

## Backend Syntax Explained

Declarch uses **backend prefix** to specify package manager:

```kdl
packages        // Default (AUR on Arch)
packages:cargo  // Rust crates
packages:npm    // Node.js packages
packages:python // Python packages
```

Each backend:
- **packages** - Installs via `paru -S` / `pacman -S`
- **packages:cargo** - Installs via `cargo install`
- **packages:npm** - Installs via `npm install -g`
- **packages:python** - Installs via `pip install`

## Editor Integration

These tools work great with:
- **Neovim/Vim** - With LSP configuration
- **VS Code** - With extensions
- **JetBrains IDEs** - Built-in support

## Organizing by Language

Keep your dev tools organized by backend:

```kdl
// General tools (system)
packages {
    neovim
    tmux
    git
}

// Language-specific
packages:cargo { /* Rust tools */ }
packages:npm { /* Node tools */ }
packages:python { /* Python tools */ }
packages:go { /* Go tools */ }
```

## Source Files

- [`development.kdl`](https://github.com/nixval/declarch/blob/main/examples/development.kdl)

---

**Next:** See [Modular Setup](modular.html) for organizing complex configurations.
