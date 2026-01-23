# Hooks Syntax Proposal - v0.6.0

## Proposed Syntax

### 1. Global Hooks (no package association)
```kdl
hooks {
    pre-sync "echo 'Starting sync...'"
    post-sync "notify-send 'Sync complete'"
    post-sync "fc-cache -fv" --sudo
}
```

### 2. Per-Package Hooks (grouped by package)
```kdl
hooks {
    docker {
        post-install "systemctl enable docker" --sudo
        post-remove "systemctl disable docker" --sudo
    }

    nvidia {
        post-install "mkinitcpio -P" --sudo
    }

    waybar {
        post-install "systemctl --user enable --now waybar.service"
    }
}
```

### 3. Shorthand Syntax (alternative to block)
```kdl
hooks {
    // Equivalent to: docker { post-install "..." }
    docker:post-install "systemctl enable docker" --sudo

    // Equivalent to: waybar { post-install "..." }
    waybar:post-install "systemctl --user enable --now waybar.service"
}
```

### 4. Conditional Hooks
```kdl
hooks {
    // Run only if package was installed/changed
    post-install "fc-cache -fv" if-changed="font-backend"

    // Run only if specific package is installed
    post-sync "systemctl restart gdm" if-installed="gdm"

    // Run only for specific backend
    post-sync "flatpak update --appstream" if-backend="flatpak"

    // Run only if previous hook succeeded
    nvidia:post-install "reboot-check.sh" if-success
}
```

### 5. Mixed Syntax (combining all)
```kdl
hooks {
    // Global hooks
    pre-sync "echo 'Starting sync...'"
    post-sync "notify-send 'Done'"

    // Per-package blocks (readable for multiple hooks)
    docker {
        post-install "systemctl enable docker" --sudo
        post-remove "docker system prune -f"
    }

    // Shorthand (good for single hook)
    waybar:post-install "systemctl --user restart waybar"

    // Conditional
    post-sync "fc-cache -fv" if-changed="font-backend"
}
```

---

## Hook Types & Flags

### Execution Flags
```kdl
--sudo     # Run with sudo (default: without sudo)
--ignore   # Ignore errors (default: warn on error)
--required # Fail if hook fails (default: warn only)
```

### Conditional Flags
```kdl
if-installed="package"    # Run only if package is installed
if-changed="package"      # Run only if package was installed/updated
if-backend="aur"          # Run only if this backend had changes
if-success                # Run only if previous hook succeeded
```

---

## Parsing Strategy

### Key Design Decisions

1. **Package detection in hooks block**:
   - `docker { ... }` → Package name is "docker"
   - `docker:post-install` → Package name is "docker", phase is "post-install"
   - `post-sync "..."` → No package, global hook

2. **No ambiguity with backends**:
   - `packages:aur { docker }` → This is in PACKAGES block
   - `hooks { docker { ... } }` → This is in HOOKS block
   - Different context = no confusion

3. **Shorthand parsing**:
   - `package:phase` → Parse as "package" + "phase"
   - But what about `nvidia:post-install "command"` vs `post-sync "command"`?
   - **Solution**: Check if contains `:` - if yes, split into package:phase

---

## Hook Phases

### Global Phases (no package context)
```kdl
hooks {
    pre-sync      // Before any package operation
    post-sync     // After all package operations
    on-success    // After successful sync
    on-failure    // After failed sync
}
```

### Package Phases (require package context)
```kdl
hooks {
    docker {
        pre-install   // Before installing docker
        post-install  // After installing docker
        pre-remove    // Before removing docker
        post-remove   // After removing docker
        on-update     // When docker is updated
    }
}
```

### Backend Phases (optional, future)
```kdl
hooks {
    aur {
        pre-sync     // Before AUR operations
        post-sync    // After AUR operations
    }
}
```

---

## Implementation Plan

### Phase 1: Core Structure (v0.6.0)

#### 1.1 Update HookEntry struct
```rust
pub struct HookEntry {
    pub command: String,
    pub hook_type: HookType,  // User or Root
    pub phase: HookPhase,     // When to run
    pub package: Option<String>,  // Which package (None = global)
    pub conditions: Vec<HookCondition>,  // When conditions
    pub error_behavior: ErrorBehavior,  // Ignore or Required
}

pub enum HookPhase {
    PreSync,
    PostSync,
    PreInstall,
    PostInstall,
    PreRemove,
    PostRemove,
    OnUpdate,
    OnSuccess,
    OnFailure,
}

pub enum HookCondition {
    IfInstalled(String),     // if-installed="package"
    IfChanged(String),       // if-changed="package"
    IfBackend(String),       // if-backend="aur"
    IfSuccess,               // if-success
}

pub enum ErrorBehavior {
    Warn,      // Default: warn on error
    Required,  // Fail sync if hook fails
    Ignore,    // Silently ignore errors
}
```

#### 1.2 Parsing in hooks block
```rust
fn parse_hooks_node(node: &KdlNode) -> Result<Vec<HookEntry>> {
    let mut hooks = Vec::new();

    if let Some(children) = node.children() {
        for child in children.nodes() {
            let name = child.name().value();

            // Case 1: Block syntax: docker { post-install "..." }
            if name.contains(':') {
                // This is shorthand: docker:post-install
                // Handled in Case 2
            } else {
                // This is package block
                parse_package_hook_block(child, &mut hooks)?;
            }
        }
    }

    Ok(hooks)
}

fn parse_package_hook_block(node: &KdlNode, hooks: &mut Vec<HookEntry>) -> Result<()> {
    let package_name = node.name().value();

    if let Some(children) = node.children() {
        for child in children.nodes() {
            let phase = child.name().value();  // post-install, etc
            let command = get_first_string(child)?;

            hooks.push(HookEntry {
                package: Some(package_name.to_string()),
                phase: parse_phase(phase)?,
                // ... rest
            });
        }
    }

    Ok(())
}
```

#### 1.3 Execution in sync.rs
```rust
// After installing each package
for pkg in packages_to_install {
    install_package(pkg)?;

    // Run post-install hooks for this package
    let package_hooks = get_hooks_for_package(&config.hooks, &pkg.name, HookPhase::PostInstall);
    execute_hooks(&package_hooks)?;
}

// After all operations
execute_hooks(&get_hooks_by_phase(&config.hooks, HookPhase::PostSync))?;
```

### Phase 2: Conditional Hooks (v0.6.1)
Add `if-changed`, `if-installed` support

### Phase 3: Backend Hooks (v0.7.0)
Add per-backend hooks support

---

## Examples

### Real-world Usage

#### Docker Service
```kdl
packages {
    docker
}

hooks {
    docker {
        post-install "systemctl enable --now docker" --sudo --required
        post-remove "systemctl disable docker" --sudo
    }
}
```

#### NVIDIA Drivers
```kdl
packages {
    nvidia
}

hooks {
    nvidia {
        post-install "mkinitcpio -P" --sudo --required
    }
}
```

#### Desktop Environment
```kdl
packages {
    gdm
    waybar
}

hooks {
    gdm:post-install "systemctl enable gdm" --sudo

    waybar {
        post-install "systemctl --user enable waybar"
        post-install "pkill -SIGUSR1 waybar"
    }

    // Global hook for font cache
    post-sync "fc-cache -fv" if-changed="font-backend" --sudo
}
```

#### Development Tools
```kdl
packages {
    rust
    nodejs
}

hooks {
    rust {
        post-install "rustup component add rust-analyzer"
    }

    nodejs:post-install "npm install -g yarn"

    // Only run if Rust was actually installed/updated
    post-sync "cargo install-update -a" if-changed="rust"
}
```

---

## Questions to Resolve

1. **Phase naming**: `post-install` vs `on-install`?
   - [ ] `post-install` (past tense, after action)
   - [ ] `on-install` (event-based)

2. **Sudo flag placement**: Before or after command?
   - [ ] After: `post-install "command" --sudo`
   - [ ] Before: `post-install --sudo "command"`

3. **Shorthand syntax**: Keep or remove?
   - [ ] Keep: `docker:post-install "command"`
   - [ ] Remove: Only use block syntax

4. **Error handling**: Default behavior?
   - [ ] Warn (current behavior)
   - [ ] Fail (stop sync on error)
   - [ ] Ignore (silently continue)

5. **Condition syntax**: `if-changed` or `if:changed`?
   - [ ] `if-changed="package"` (flat)
   - [ ] `if changed="package"` (separate node)
