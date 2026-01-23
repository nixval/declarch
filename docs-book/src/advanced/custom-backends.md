# Custom Backends

Extend declarch with custom package manager backends.

## What Are Custom Backends?

Custom backends allow you to define support for package managers that aren't built into declarch. This is useful for:

- Distro-specific package managers (dnf, nala, apt, etc.)
- Language package managers not included (go, composer, etc.)
- Internal/private package managers
- Experimental or custom package management systems

## Backend Definition

Backends are defined in your declarch configuration using the `backends` block:

```kdl
backends {
    <backend-name> {
        cmd "<install-command>"
        list_cmd "<list-command>"
        remove_cmd "<remove-command>"
    }
}
```

## Required Fields

| Field | Description | Example |
|-------|-------------|---------|
| `cmd` | Install command template | `"sudo nala install"` |
| `list_cmd` | List installed packages | `"nala list --installed"` |

## Optional Fields

| Field | Description | Example |
|-------|-------------|---------|
| `remove_cmd` | Remove command template | `"sudo nala remove"` |
| `noconfirm_flag` | Skip confirmation flag | `"-y"` |
| `update_cmd` | Update command | `"sudo nala update"` |

## Examples

### Nala (Debian/Ubuntu)

```kdl
backends {
    nala {
        cmd "sudo nala install -y"
        list_cmd "dpkg -l | awk '/^ii/ {print $2}'"
        remove_cmd "sudo nala remove -y"
        noconfirm_flag "-y"
    }
}

// Use the backend
packages:nala {
    vim
    ffmpeg
    imagemagick
}
```

### DNF (Fedora)

```kdl
backends {
    dnf {
        cmd "sudo dnf install -y"
        list_cmd "rpm -qa --queryformat='%{NAME}\n'"
        remove_cmd "sudo dnf remove -y"
        noconfirm_flag "-y"
    }
}

packages:dnf {
    vim
    ffmpeg
}
```

### apt (Debian/Ubuntu)

```kdl
backends {
    apt {
        cmd "sudo apt install -y"
        list_cmd "dpkg -l | awk '/^ii/ {print $2}'"
        remove_cmd "sudo apt remove -y"
        noconfirm_flag "-y"
    }
}

packages:apt {
    vim
    curl
    wget
}
```

### Go

```kdl
backends {
    go {
        cmd "go install"
        list_cmd "ls ~/go/bin/"
        remove_cmd "rm ~/go/bin/<package>"
    }
}

packages:go {
    github.com/cli/cli@latest
    golang.org/x/tools/gopls@latest
}
```

### Composer (PHP)

```kdl
backends {
    composer {
        cmd "composer global require"
        list_cmd "composer global show --name-only"
        remove_cmd "composer global remove"
    }
}

packages:composer {
    laravel/installer
    phpstan/phpstan
}
```

### Ruby Gems

```kdl
backends {
    gem {
        cmd "gem install"
        list_cmd "gem list --no-versions"
        remove_cmd "gem uninstall"
        noconfirm_flag "--no-document"
    }
}

packages:gem {
    bundler
    rake
}
```

## Command Placeholders

Commands support the following placeholder:

| Placeholder | Replaced With |
|-------------|---------------|
| `<package>` | Package name |

### Example with Placeholders

```kdl
backends {
    custom {
        cmd "install-package --name <package>"
        remove_cmd "remove-package --name <package>"
    }
}
```

During sync:
```bash
# For package "vim"
install-package --name vim

# For removal
remove-package --name vim
```

## Complete Backend Example

### Pacman (Arch - Alternative Implementation)

```kdl
backends {
    pacman {
        cmd "sudo pacman -S --noconfirm"
        list_cmd "pacman -Qq"
        remove_cmd "sudo pacman -Rns --noconfirm"
        noconfirm_flag "--noconfirm"
        update_cmd "sudo pacman -Syu"
    }
}

packages:pacman {
    vim
    bat
    exa
}
```

### Xbps (Void Linux)

```kdl
backends {
    xbps {
        cmd "sudo xbps-install -y"
        list_cmd "xbps-query -l | awk '{print $2}'"
        remove_cmd "sudo xbps-remove -y"
        noconfirm_flag "-y"
    }
}

packages:xbps {
    vim
    curl
}
```

### Zypper (openSUSE)

```kdl
backends {
    zypper {
        cmd "sudo zypper install -y"
        list_cmd "zypper se --installed-only | awk '{print $3}'"
        remove_cmd "sudo zypper remove -y"
        noconfirm_flag "-y"
    }
}

packages:zypper {
    vim
    tmux
}
```

## Advanced Features

### Multiple Commands per Backend

For complex installation processes, use a script:

```kdl
backends {
    complex {
        cmd "~/.config/declarch/backends/complex-backend.sh install <package>"
        list_cmd "~/.config/declarch/backends/complex-backend.sh list"
        remove_cmd "~/.config/declarch/backends/complex-backend.sh remove <package>"
    }
}
```

### Backend-Specific Options

```kdl
options:apt {
    noconfirm
}

options:dnf {
    noconfirm
    setopt "install_weak_deps=False"
}
```

### Environment Variables per Backend

```kdl
env:dnf DNF_ASSUME_YES="1"
env:apt DEBIAN_FRONTEND="noninteractive"
```

## Backend Script Example

Create a backend script for complex logic:

```bash
#!/bin/bash
# ~/.config/declarch/backends/custom-backend.sh

BACKEND_DIR="$HOME/.custom-packages"
mkdir -p "$BACKEND_DIR"

case "$1" in
    install)
        pkg="$2"
        echo "Installing $pkg..."
        # Custom installation logic
        wget -O "$BACKEND_DIR/$pkg" "https://example.com/$pkg"
        chmod +x "$BACKEND_DIR/$pkg"
        ;;
    list)
        ls -1 "$BACKEND_DIR"
        ;;
    remove)
        pkg="$2"
        echo "Removing $pkg..."
        rm -f "$BACKEND_DIR/$pkg"
        ;;
    *)
        echo "Usage: $0 {install|list|remove} [package]"
        exit 1
        ;;
esac
```

Make it executable:
```bash
chmod +x ~/.config/declarch/backends/custom-backend.sh
```

Use in config:
```kdl
backends {
    custom {
        cmd "~/.config/declarch/backends/custom-backend.sh install <package>"
        list_cmd "~/.config/declarch/backends/custom-backend.sh list"
        remove_cmd "~/.config/declarch/backends/custom-backend.sh remove <package>"
    }
}
```

## Testing Custom Backends

### 1. Define Backend

```kdl
backends {
    test {
        cmd "echo 'Installing: <package>'"
        list_cmd "echo 'installed1\ninstalled2'"
        remove_cmd "echo 'Removing: <package>'"
    }
}
```

### 2. Check Backend

```bash
declarch check --backend test
```

### 3. Dry Run Sync

```bash
packages:test {
    test-package
}

declarch sync --dry-run
```

### 4. Test Install

```bash
declarch sync --target test
```

## Troubleshooting

### Backend Not Recognized

**Problem:** `declarch check` doesn't show your backend

**Solution:**
```bash
# Check syntax
declarch check

# Verify backend block exists
cat ~/.config/declarch/declarch.kdl | grep -A 10 "backends"
```

### Command Not Found

**Problem:** Backend command fails with "command not found"

**Solution:**
```bash
# Test command manually
sudo nala --version

# Use full path if needed
backends {
    nala {
        cmd "/usr/bin/nala install -y"
    }
}
```

### List Command Returns Wrong Format

**Problem:** Packages not recognized

**Solution:** Ensure list command returns one package per line:
```bash
# Test list command
dpkg -l | awk '/^ii/ {print $2}'

# Should output:
# package1
# package2
# package3
```

### Permission Issues

**Problem:** Backend can't install without sudo

**Solution:** Include sudo in command:
```kdl
backends {
    custom {
        cmd "sudo custom-pkg install <package>"
    }
}
```

## Best Practices

### 1. Use Absolute Paths

```kdl
backends {
    custom {
        cmd "/usr/bin/custom-install"
    }
}
```

### 2. Handle Confirmation Prompts

```kdl
backends {
    apt {
        cmd "sudo apt install -y"  # -y skips prompts
        noconfirm_flag "-y"
    }
}
```

### 3. Test List Command Thoroughly

```bash
# Your list command should work like this:
$ your-list-cmd
package1
package2
package3

# Not like this:
$ your-list-cmd
package1 version1
package2 version2  # Wrong format
```

### 4. Use Scripts for Complex Logic

For complex backends, create a script rather than embedding complex commands:

```kdl
backends {
    custom {
        cmd "~/.config/declarch/backends/custom.sh install"
    }
}
```

### 5. Document Your Backend

```kdl
// Custom backend for MyPackage Manager
// Requires: mypkg-cli >= 2.0
// Repository: https://github.com/example/mypkg

backends {
    mypkg {
        cmd "mypkg install"
        list_cmd "mypkg list"
        remove_cmd "mypkg remove"
    }
}
```

## Submitting Backends

If you create a generally useful backend, consider contributing it to declarch:

1. Test thoroughly on multiple systems
2. Document requirements and installation
3. Open an issue or pull request on GitHub

## Related

- [Backends Reference](../configuration/backends.md) - Built-in backends
- [KDL Syntax Reference](../configuration/kdl-syntax.md) - Configuration syntax
- [Troubleshooting](troubleshooting.md) - Common issues

## Examples Gallery

### Distro Package Managers

```kdl
// Debian/Ubuntu
backends {
    apt { ... }
}

// Fedora
backends {
    dnf { ... }
}

// Arch (alternative)
backends {
    pacman { ... }
}
```

### Language Package Managers

```kdl
// Go
backends {
    go { ... }
}

// PHP
backends {
    composer { ... }
}

// Ruby
backends {
    gem { ... }
}
```

### Custom/Private

```kdl
// Company internal
backends {
    company-pkg {
        cmd "company-pkg-client install"
        list_cmd "company-pkg-client list"
        remove_cmd "company-pkg-client remove"
    }
}
```
