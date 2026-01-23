# Hooks System - Brainstorming & Design

## Current State Analysis

### Existing Hook Types (kdl.rs:136-144)
```rust
pub enum HookType {
    Run,        // Run without sudo
    SudoNeeded, // Explicitly needs sudo
    Script,     // Run a script file (NOT IMPLEMENTED)
    Backup,     // Backup a file (NOT IMPLEMENTED)
    Notify,     // Send notification (NOT IMPLEMENTED)
}
```

### Current Syntax (3 types)

#### 1. Inline flat hooks (most common)
```kdl
on-sync "notify-send 'Packages updated'"
on-sync-sudo "systemctl restart gdm"
on-pre-sync "echo 'Starting sync...'"
```

#### 2. hooks block (structured, but underutilized)
```kdl
hooks {
    on-sync "notify-send 'Updated'"
    on-sync-sudo "systemctl restart gdm"
}
```

#### 3. Package-level hooks (mentioned in docs, need to verify)
```kdl
packages {
    docker enable=true post-install="systemctl enable docker"
}
```

### Current Implementation
- **Execution**: `src/commands/hooks.rs`
- **Parsing**: `src/config/kdl.rs` (lines 610-639)
- **Trigger**: `src/commands/sync.rs` (pre-sync: line 118, post-sync: line 369)
- **Security**: Requires `--hooks` flag to execute (line 97 in sync.rs)
- **Error handling**: Warnings only, doesn't fail sync

---

## Problems & Limitations

### 1. **Ambiguous Hook Types**
- `HookType` enum has `Script`, `Backup`, `Notify` but they're never used
- Only `Run` and `SudoNeeded` are actually implemented
- Confusing for users and developers

### 2. **No Per-Package Hooks**
- Mentioned in docs but not implemented
- Can't attach hooks to specific packages
- Example: `docker enable=true post-install="systemctl enable docker"`

### 3. **No Backend-Specific Hooks**
- Can't run hooks only for AUR packages, or only for Flatpak
- All hooks run globally regardless of what changed

### 4. **No Conditional Hooks**
- Can't run hooks based on:
  - Whether specific packages were installed/updated
  - Backend type
  - Exit status of previous hook
  - System state

### 5. **Limited Hook Phases**
Only 2 phases:
- `pre-sync` - before any package operations
- `post-sync` - after all package operations

Missing phases:
- `pre-install` - before each package install
- `post-install` - after each package install
- `pre-remove` - before each package remove
- `post-remove` - after each package remove
- `on-failure` - when sync fails
- `on-success` - when sync succeeds

### 6. **No Hook Organization**
- All hooks are flat in one list
- Can't group related hooks
- Can't name/categorize hooks

### 7. **Security Model is Basic**
- Binary: all or nothing (`--hooks` flag)
- Can't:
  - Trust specific sources
  - Review hooks interactively
  - Dry-run with hooks
  - Sandbox hooks

### 8. **No Hook Context/Variables**
- Can't access:
  - Which packages were installed
  - Which backend was used
  - Exit codes
  - Environment info

---

## Brainstorming Solutions

### A. Simplify Hook Types (Keep Simple Approach)

**Proposal**: Remove unused types, add clarity

```rust
pub enum HookType {
    Normal,     // Regular command without sudo
    Sudo,       // Command that needs sudo
}
```

**OR** keep it more explicit:

```rust
pub enum HookType {
    User,       // Run as current user
    Root,       // Run with sudo
}
```

### B. Add Per-Package Hooks

#### Option 1: Inline attributes (current syntax mentioned)
```kdl
packages {
    docker enable=true post-install="systemctl enable docker"
    nvidia post-remove="rm /etc/modprobe.d/nvidia.conf"
}
```

#### Option 2: Nested hooks block
```kdl
packages {
    docker {
        hooks {
            post-install "systemctl enable docker"
            post-remove "systemctl disable docker"
        }
    }
}
```

#### Option 3: Separate hook definitions
```kdl
hooks {
    on-package-install docker "systemctl enable docker"
    on-package-remove docker "systemctl disable docker"
}
```

### C. Add Conditional Hooks

#### Option 1: Inline conditions
```kdl
on-sync "systemctl restart gdm" if-installed="gdm"
on-sync "fc-cache -fv" if-changed="font-backend"
```

#### Option 2: Structured conditions
```kdl
hooks {
    on-sync "systemctl restart gdm" {
        condition "package-installed" {
            package "gdm"
        }
    }
}
```

#### Option 3: Event-based hooks
```kdl
hooks {
    on-event "package:installed:nvidia" {
        "mkinitcpio -P"
    }
    on-event "backend:completed:flatpak" {
        "flatpak update --appstream"
    }
}
```

### D. Add More Hook Phases

```kdl
hooks {
    // Global phases
    pre-sync "echo 'Starting...'"
    post-sync "notify-send 'Done'"

    // Per-package phases
    pre-install "echo 'Installing...'"
    post-install "update-desktop-database"

    // Per-backend phases
    pre-aur "echo 'Updating AUR...'"
    post-flatpak "flatpak update --appstream"

    // Error handling
    on-failure "notify-send 'Sync failed'"
    on-success "notify-send 'Sync succeeded'"
}
```

### E. Organize Hooks with Names/Groups

```kdl
hooks {
    group "desktop" {
        description "Desktop environment hooks"
        post-sync "systemctl restart gdm"
        post-sync "pkill -SIGUSR1 waybar"
    }

    group "fonts" {
        description "Font cache hooks"
        post-sync "fc-cache -fv"
    }
}
```

### F. Improved Security Model

#### Option 1: Trust-based
```kdl
// In declarch.kdl (root config)
trusted-sources "github.com/nixval/*" "gitlab.com/myuser/*"

// In remote modules
hooks {
    // Trusted source - runs without --hooks flag
    post-sync "notify-send 'Updated'"
}
```

#### Option 2: Interactive review
```bash
$ dcl sync --review-hooks

⚠️  Remote module 'desktop' has hooks:
  1. systemctl restart gdm [requires sudo]

Execute these hooks? [Y/n]
```

#### Option 3: Per-module trust
```kdl
imports {
    "modules/base.kdl"           // Local - always trusted
    "github.com/user/desktop.kdl" // Remote - needs --hooks
    "github.com/nixval/dev.kdl"   // Trusted - always allowed
}

trusted-modules "github.com/nixval/*"
```

### G. Hook Context/Variables

#### Option 1: Environment variables
```kdl
on-sync "notify-send 'Installed: $DCL_PACKAGES'"
```

#### Option 2: Template expansion
```kdl
on-sync "notify-send 'Installed: {{packages}}'"
```

#### Option 3: Script files with context
```kdl
on-sync "~/.config/declarch/hooks/post-sync.sh"
```

```bash
#!/bin/bash
# post-sync.sh - has access to environment variables
echo "Installed: $DCL_INSTALLED_PACKAGES"
echo "Updated: $DCL_UPDATED_PACKAGES"
echo "Backend: $DCL_BACKEND"
```

---

## Proposed Syntax Options

### Option 1: Keep Current + Add Per-Package Hooks (Minimal)
```kdl
// Current global hooks (keep as-is)
on-sync "notify-send 'Updated'"
on-sync-sudo "systemctl restart gdm"

// New: Per-package hooks
packages {
    docker post-install="systemctl enable docker"
    nvidia post-install="mkinitcpio -P"
}
```

**Pros**: Simple, backward compatible
**Cons**: Limited flexibility

### Option 2: Unified hooks Block (Structured)
```kdl
hooks {
    // Global hooks
    post-sync "notify-send 'Updated'"

    // Per-package hooks
    on-package-install docker "systemctl enable docker"
    on-package-install nvidia "mkinitcpio -P"

    // Conditional hooks
    post-sync "fc-cache -fv" if-backend="flatpak"
}
```

**Pros**: More organized, explicit
**Cons**: More verbose

### Option 3: Event-Based System (Advanced)
```kdl
hooks {
    // Event-driven
    on "sync:complete" {
        "notify-send 'Done'"
        "pkill waybar"
    }

    on "package:installed" {
        if "docker" {
            "systemctl enable docker"
        }
    }

    on "backend:complete" {
        if "flatpak" {
            "flatpak update --appstream"
        }
    }
}
```

**Pros**: Most flexible, powerful
**Cons**: Complex, learning curve

---

## Recommended Approach

### Phase 1: Cleanup (Immediate)
1. Remove unused `HookType` variants (`Script`, `Backup`, `Notify`)
2. Rename `Run` → `User`, `SudoNeeded` → `Root`
3. Document current behavior clearly

### Phase 2: Per-Package Hooks (v0.6.0)
```kdl
packages {
    docker post-install="systemctl enable docker"
    nvidia post-install="mkinitcpio -P"
}
```

### Phase 3: Conditional Hooks (v0.7.0)
```kdl
hooks {
    post-sync "fc-cache -fv" if-changed="fonts"
    post-sync "systemctl restart gdm" if-installed="gdm"
}
```

### Phase 4: Organized Hooks (v0.8.0)
```kdl
hooks {
    group "desktop" {
        post-sync "systemctl restart gdm"
        post-sync "pkill waybar"
    }
}
```

### Phase 5: Advanced Security (v1.0.0)
```kdl
trusted-sources "github.com/nixval/*"
```

---

## Questions for Discussion

1. **Per-package hooks**: Which syntax do you prefer?
   - [ ] Inline: `docker post-install="systemctl enable docker"`
   - [ ] Nested: `docker { hooks { post-install "..." } }`
   - [ ] Separate: `on-package-install docker "..."`

2. **Conditional hooks**: Do we need this?
   - [ ] Yes, if-installed/if-changed are essential
   - [ ] No, keep it simple for now
   - [ ] Maybe, add in later phase

3. **Hook organization**: Groups or flat?
   - [ ] Groups for organization
   - [ ] Keep flat and simple

4. **Security model**: How to handle remote hooks?
   - [ ] Current: --hooks flag for everything
   - [ ] Trust-based: Trusted sources don't need flag
   - [ ] Interactive: Review hooks before running

5. **Hook types**: Simplify or keep as-is?
   - [ ] Simplify to User/Root only
   - [ ] Keep Run/SudoNeeded (rename them)
   - [ ] Add Script type for script files
