# Development Environment Setup

Configuration for software development.

## Quick Example

```kdl
meta {
  host "dev-machine"
  description "Development computer"
}

// Core tools
packages {
  neovim
  git
  docker
  gh       // GitHub CLI
}

// Rust tools
packages:cargo {
  ripgrep    // Fast search
  fd-find    // Fast find
  zoxide     // Smart cd
  bat       // Better cat
}

// Node.js tools
packages:npm {
  typescript
  prettier
  eslint
  vite
}

// Python tools
packages:python {
  black     // Formatter
  ruff      // Linter
  jupyter   // Notebooks
}
```

## What This Includes

### Core Tools (System)
- **Neovim** - Text editor
- **Git** - Version control
- **Docker** - Containers
- **gh** - GitHub CLI

### Language-Specific Tools

**Rust (via cargo):**
- ripgrep - Fast text search
- fd-find - Fast file search
- zoxide - Smart directory navigation
- bat - Enhanced cat

**Node.js (via npm):**
- TypeScript - Type-safe JavaScript
- Prettier - Code formatter
- ESLint - Linter
- Vite - Fast build tool

**Python (via pip):**
- black - Code formatter
- ruff - Fast linter
- Jupyter - Interactive notebooks

## Package Managers Supported

Declarch works with many package managers:

| Backend | Command | Example Packages |
|---------|---------|-----------------|
| `packages` | paru/pacman | neovim, git, docker |
| `packages:cargo` | cargo install | ripgrep, fd-find |
| `packages:npm` | npm install -g | typescript, prettier |
| `packages:python` | pip install | black, ruff |
| `packages:go` | go install | (custom backend) |
| `packages:flatpak` | flatpak install | IDEs and apps |

## Three Syntax Styles

Choose the style you prefer:

**Style 1: Backend blocks** (recommended):
```kdl
packages:npm {
  typescript
  prettier
}

packages:cargo {
  ripgrep
}
```

**Style 2: Embedded blocks**:
```kdl
packages {
  npm {
    typescript
    prettier
  }

  cargo {
    ripgrep
  }
}
```

**Style 3: Inline**:
```kdl
packages {
  npm:typescript
  npm:prettier
  cargo:ripgrep
}
```

## Development Workflow

```bash
# Setup
declarch sync

# Daily work
rg "function"     # Search code (ripgrep)
fd "test.rs"      # Find files (fd)
zoxide proj       # Jump to project (zoxide)

# JavaScript
npm init vite     # Create new project

# Python
jupyter notebook  # Start notebook
```

## Source Files

- [`development.kdl`](https://github.com/nixval/declarch/blob/main/examples/development.kdl)

---

**Next:** See [Modular Setup](modular.html) to organize configs.
