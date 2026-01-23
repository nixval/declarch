# Development Environment Setup

Configuration optimized for software development.

## Configuration

```kdl
// ~/.config/declarch/declarch.kdl

meta {
  host "dev-machine"
  tags "development" "programming"
}

// Core development tools
packages:cargo {
  // CLI tools
  ripgrep      # Fast grep alternative
  fd-find      # Fast find alternative
  zoxide       # Smart cd
  bat          # Better cat
  eza          # Better ls
  xh           # HTTP client
  mdbook       # Book creation

  // Build tools
  cargo-edit   # Cargo extensions
}

packages:npm {
  # JavaScript/TypeScript
  typescript
  prettier
  eslint
  vite
  create-vite

  # AWS SDK
  @aws-sdk/client-s3
}

packages:pip {
  # Python tools
  python-black
  mypy
  pytest
  pre-commit
}

// Language servers
packages {
  # System packages
  go
  nodejs
  python
  rustup
  jdk-openjdk
}

// Optional: Docker and containers
packages {
  docker
  docker-compose
}
```

## What This Includes

### Rust Development
- **ripgrep** - Fast text search
- **fd** - Fast file search
- **zoxide** - Smart directory navigation
- **bat** - Enhanced cat with syntax highlighting
- **eza** - Enhanced ls with colors
- **xh** - Modern HTTP client
- **mdbook** - Create books from markdown files

### JavaScript/TypeScript
- **TypeScript** - Type-safe JavaScript
- **Prettier** - Code formatter
- **ESLint** - Linter
- **Vite** - Fast build tool
- **create-vite** - Project scaffolding

### Python
- **black** - Code formatter
- **mypy** - Type checker
- **pytest** - Testing framework
- **pre-commit** - Git hooks framework

### Multiple Languages
- **Go** - Google's Go language
- **Node.js** - JavaScript runtime
- **Python** - General purpose language
- **Rust** - Systems language
- **OpenJDK** - Java development kit

### Container Tools
- **Docker** - Container runtime
- **Docker Compose** - Multi-container apps

## Development Workflow

```bash
# Initial setup
declarch init
declarch sync

# Start new project
mkdir my-project
cd my-project
npm init vite  # Via npm packages

# Code with tools
rg "function"  # ripgrep search
fd "test.rs"   # fd find files
zoxide proj    # zoxide jump to project

# Run and test
cargo run      # Rust
npm test       # Node
pytest         # Python
```

## Editor Integration

These tools work great with:
- **Neovim/Vim** - With LSP configuration
- **VS Code** - With extensions
- **JetBrains IDEs** - Built-in support

## Source Files

- [`development.kdl`](https://github.com/nixval/declarch/blob/main/examples/development.kdl)

---

**Next:** See [Modular Setup](modular.html) for organizing complex configurations.
