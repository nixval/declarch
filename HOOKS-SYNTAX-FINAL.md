# Hooks Syntax - Final Decision (v0.6.0)

## Syntax Specifications

### 1. Hook Phases
**Decision**: Use past tense (`post-install`, not `on-install`)

### 2. Sudo Flag Placement
**Decision**: AFTER command
```kdl
docker:post-install "systemctl enable docker" --sudo

// Tools will:
// - Strip "sudo" from inside the command string
// - Use the --sudo flag to determine execution mode
// - Error if user puts "sudo" inside the command
```

**Validation**:
```rust
// If command contains "sudo", error:
if command.starts_with("sudo ") {
    return Err("Use --sudo flag instead of embedding 'sudo' in command");
}

// Execute with --sudo flag:
if has_sudo_flag {
    Command::new("sudo").arg(&command).args(&args);
}
```

### 3. Shorthand Syntax
**Decision**: KEEP both block and shorthand
```kdl
// Block (good for multiple hooks)
docker {
    post-install "systemctl enable docker" --sudo
    post-remove "docker system prune -f"
}

// Shorthand (good for single hook)
waybar:post-install "systemctl --user restart waybar"
```

### 4. Error Handling
**Decision**:
- **Hooks**: Default WARN (don't fail sync)
- **Package installation**: Default FAIL (stop sync on error)

```kdl
hooks {
    // Hook errors: warn by default
    docker:post-install "systemctl enable docker" --sudo

    // Or require success:
    docker:post-install "mkinitcpio -P" --sudo --required

    // Or ignore errors:
    docker:post-install "cleanup.sh" --ignore
}
```

### 5. Conditional Syntax
**Decision**: Node-based (extensible), with shorthand support

#### Shorthand (common case)
```kdl
post-sync "fc-cache -fv" if changed="font-backend"
post-sync "systemctl restart gdm" if installed="gdm"
```

#### Full syntax (future-proof)
```kdl
post-sync "fc-cache -fv" {
    if changed="font-backend"
}

// Can be extended later:
post-sync "notify-send 'Done'" {
    if changed="docker" {
        else "notify-send 'Nothing changed'"
    }
}
```

---

## Complete Grammar

### Global Hooks
```kdl
hooks {
    pre-sync "command" [--sudo] [--required] [--ignore] [if condition]
    post-sync "command" [--sudo] [--required] [--ignore] [if condition]
    on-success "command" [--sudo] [--required] [--ignore] [if condition]
    on-failure "command" [--sudo] [--required] [--ignore] [if condition]
}
```

### Package Hooks (Block)
```kdl
hooks {
    <package-name> {
        pre-install "command" [flags] [condition]
        post-install "command" [flags] [condition]
        pre-remove "command" [flags] [condition]
        post-remove "command" [flags] [condition]
        on-update "command" [flags] [condition]
    }
}
```

### Package Hooks (Shorthand)
```kdl
hooks {
    <package>:<phase> "command" [flags] [condition]
}
```

### Flags
```kdl
--sudo      # Execute with sudo
--required  # Fail sync if hook fails
--ignore    # Silently ignore errors
```

### Conditions (Shorthand)
```kdl
if changed="package"     # Run only if package was installed/updated
if installed="package"   # Run only if package exists
if backend="aur"         # Run only if this backend had changes
if success               # Run only if previous hook succeeded
```

### Conditions (Full)
```kdl
post-sync "command" {
    if changed="package" {
        and backend="aur"    // Can add more conditions later
        else "fallback-cmd"  // Can add else later
    }
}
```

---

## Implementation Priority

### Phase 1: Core (v0.6.0)
- [ ] Update data structures (HookEntry, HookPhase, HookCondition)
- [ ] Parse global hooks (pre-sync, post-sync, on-success, on-failure)
- [ ] Parse package hooks (block syntax)
- [ ] Parse package hooks (shorthand syntax)
- [ ] Implement --sudo flag (with validation)
- [ ] Execute hooks in sync process
- [ ] Error handling: warn by default

### Phase 2: Conditions (v0.6.1)
- [ ] Parse `if changed="package"` (shorthand)
- [ ] Parse `if installed="package"` (shorthand)
- [ ] Implement condition evaluation
- [ ] Track which packages were changed during sync

### Phase 3: Advanced Conditions (v0.7.0)
- [ ] Parse `if backend="aur"` (shorthand)
- [ ] Implement full node-based conditions
- [ ] Add `else` support
- [ ] Add compound conditions (`and`, `or`)

### Phase 4: Error Modes (v0.6.0)
- [ ] Implement --required flag
- [ ] Implement --ignore flag
- [ ] Ensure package installation still fails on error

---

## Examples

### Docker Service
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

### NVIDIA Drivers
```kdl
packages {
    nvidia
}

hooks {
    nvidia:post-install "mkinitcpio -P" --sudo --required
}
```

### Desktop Environment
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

    // Only run if fonts changed
    post-sync "fc-cache -fv" if changed="font-backend" --sudo
}
```

### Development Tools
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

    // Only run if Rust was updated
    post-sync "cargo install-update -a" if changed="rust"
}
```

### With Error Handling
```kdl
hooks {
    // Critical: fail if this errors
    nvidia:post-install "mkinitcpio -P" --sudo --required

    // Optional: warn if this errors
    docker:post-install "systemctl enable docker" --sudo

    // Non-critical: ignore errors
    cleanup:post-install "rm -rf /tmp/cache" --ignore
}
```

---

## Edge Cases & Validation

### 1. Sudo in command string
```kdl
// ERROR: Don't embed sudo in command
docker:post-install "sudo systemctl enable docker" --sudo

// Correct: Use --sudo flag
docker:post-install "systemctl enable docker" --sudo
```

**Validation**:
```rust
if command.trim().starts_with("sudo ") {
    return Err(HookError::EmbeddedSudo {
        hint: "Use --sudo flag instead of embedding 'sudo' in command"
    });
}
```

### 2. Package not found
```kdl
// WARNING: Package 'nonexistent' not in packages list
hooks {
    nonexistent:post-install "echo 'This will warn but not fail'"
}
```

### 3. Duplicate hooks
```kdl
// Both will run (order: declaration order)
hooks {
    docker:post-install "echo 'First'"
    docker:post-install "echo 'Second'"
}
```

### 4. Conditional without package context
```kdl
// OK: Global hook with condition
post-sync "fc-cache -fv" if changed="font-backend"

// ERROR: Condition references package but no package context
// (This should be in a package block)
docker:post-sync "echo 'This is weird'"
```

---

## Testing Strategy

### Unit Tests
```rust
#[test]
fn test_parse_global_hook() {
    let kdl = r#"
    hooks {
        post-sync "notify-send 'Done'"
    }
    "#;
    // Assert: parsed correctly
}

#[test]
fn test_parse_package_hook_block() {
    let kdl = r#"
    hooks {
        docker {
            post-install "systemctl enable docker" --sudo
        }
    }
    "#;
    // Assert: parsed with package=docker, phase=PostInstall, sudo=true
}

#[test]
fn test_parse_shorthand() {
    let kdl = r#"
    hooks {
        docker:post-install "systemctl enable docker" --sudo
    }
    "#;
    // Assert: same result as block syntax
}

#[test]
fn test_reject_embedded_sudo() {
    let kdl = r#"
    hooks {
        docker:post-install "sudo systemctl enable docker" --sudo
    }
    "#;
    // Assert: returns error
}
```

### Integration Tests
```bash
# Test hook execution
dcl sync --dry-run --hooks
dcl sync --hooks

# Test conditional hooks
dcl check --verbose

# Test error modes
dcl sync --hooks  # Should warn on hook error
dcl sync --hooks  # Should fail if --required
```

---

## Migration Path

### For existing users
Current syntax is still supported:
```kdl
// Still works
on-sync "notify-send 'Done'"
on-sync-sudo "systemctl restart gdm"

// New syntax (recommended)
hooks {
    post-sync "notify-send 'Done'"
    post-sync "systemctl restart gdm" --sudo
}
```

Deprecation plan:
- v0.6.0: Both syntaxes supported, warning for old syntax
- v0.7.0: Old syntax deprecated, error by default
- v1.0.0: Old syntax removed
