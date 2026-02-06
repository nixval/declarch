# Declarch v0.6 Refactor Plan: Pure Generic Core

**Branch:** `refactor/v0.6-generic-core`  
**Target:** Complete architectural shift to pure generic backend system  
**Philosophy:** Nix-style explicit imports, zero hardcoded backends

---

## 🎯 Goals

1. **Zero Hardcoded Backends** - No `Backend::Aur`, `Backend::Npm`, etc. in enum
2. **Pure Generic Manager** - Single `GenericManager` for all package operations
3. **Nix-style Config** - Explicit imports, declarative, no magic
4. **Clean Syntax** - `pkg { backend { package } }` 
5. **Fallback Support** - Simple binary fallback: `fallback "alternate-binary"`
6. **Auto-init** - `declarch init --backend <name>` auto-imports to `backends.kdl`

---

## 📋 New Architecture Overview

### File Structure

```
~/.config/declarch/
├── declarch.kdl          # Main config
├── backends.kdl          # Backend index (import all backend defs)
├── state.json            # Runtime state
└── backends/             # Backend definitions
    ├── paru.kdl
    ├── yay.kdl
    ├── pacman.kdl
    ├── npm.kdl
    ├── cargo.kdl
    └── ...
```

### Config Syntax

```kdl
// declarch.kdl
import "backends.kdl"

meta {
  name "My System"
  author "user"
}

pkg {
  paru {
    vim
    git
    htop
  }
  
  npm {
    typescript
    eslint
  }
}
```

```kdl
// backends.kdl (index file)
import "backends/paru.kdl"
import "backends/npm.kdl"
import "backends/cargo.kdl"
```

```kdl
// backends/paru.kdl
backend "paru" {
  binary "paru"
  fallback "pacman"  // Simple fallback
  
  list_cmd "paru -Q"
  install_cmd "paru -S --needed {packages}"
  remove_cmd "paru -Rns {packages}"
  
  format "regex"
  list_regex "^(\\S+)\\s+(\\S+)"
  name_group 1
  version_group 2
}
```

---

## 🚧 Phase-by-Phase Implementation

### Phase 1: Core Type System Refactor
**Duration:** 3-5 days  
**Branch:** `refactor/v0.6-generic-core/phase1-types`

#### Tasks:
1. **Rewrite `Backend` enum**
   - Remove all hardcoded variants (`Aur`, `Npm`, etc.)
   - Keep only: `Backend::Generic(String)` or `Backend::Dynamic(String)`
   - Display impl: use string directly
   - FromStr impl: accept any string

2. **Update `PackageId`**
   - Keep structure: `{ name, backend }`
   - `backend` field becomes `String` (or `Backend::Generic`)

3. **Update all type references**
   - Search/replace `Backend::Aur` → `Backend::from("aur")`
   - Fix pattern matches
   - Update tests

#### Files to Modify:
- `src/core/types.rs` - Complete rewrite
- `src/core/resolver.rs` - Update backend comparisons
- `src/core/matcher.rs` - Update variant matching
- All test files

#### Success Criteria:
```rust
// Should work:
let backend = Backend::from("anything");
assert_eq!(backend.to_string(), "anything");
```

---

### Phase 2: Config Syntax Parser
**Duration:** 5-7 days  
**Branch:** `refactor/v0.6-generic-core/phase2-parser`

#### Tasks:
1. **New `pkg` block parser**
   - Parse `pkg { backend { packages... } }`
   - Support nested blocks
   - Support package arguments (version, options)

2. **Backend block definition parser**
   - Parse `backend "name" { ... }`
   - Support all `BackendConfig` fields
   - Support `fallback` field

3. **Import statement enhancement**
   - Support `"backends/*.kdl"` glob patterns (optional)
   - Better error messages for missing imports

4. **Backward compat (temporary)**
   - Keep `packages { }` parsing but emit deprecation warning
   - Migration helper script

#### Files to Modify:
- `src/config/kdl_modules/parser.rs` - Major rewrite
- `src/config/kdl_modules/types.rs` - Add new types
- `src/config/loader.rs` - Update loading logic

#### New Types:
```rust
pub struct PackageBlock {
    pub backend: String,
    pub packages: Vec<PackageEntry>,
}

pub struct BackendDefinition {
    pub name: String,
    pub binary: String,
    pub fallback: Option<String>,
    pub list_cmd: String,
    pub install_cmd: String,
    pub remove_cmd: String,
    pub format: OutputFormat,
    // ... etc
}
```

---

### Phase 3: GenericManager Enhancement
**Duration:** 4-6 days  
**Branch:** `refactor/v0.6-generic-core/phase3-generic`

#### Tasks:
1. **Fallback logic**
   - Try primary binary
   - If not found, try fallback
   - Error only if both fail

2. **Command templating**
   - Support `{packages}` placeholder
   - Support `{binary}` placeholder for self-reference
   - Variable substitution

3. **Error handling**
   - Better error messages with backend context
   - Distinguish between "binary not found" vs "command failed"

4. **Remove old backend code**
   - Delete `src/packages/aur.rs`
   - Delete `src/packages/flatpak.rs`
   - Delete `src/packages/soar.rs`
   - Delete `src/packages/npm.rs`
   - Delete `src/packages/yarn.rs`
   - Delete `src/packages/pnpm.rs`
   - Delete `src/packages/bun.rs`
   - Delete `src/packages/cargo.rs`
   - Delete `src/packages/brew.rs`
   - Keep only `traits.rs` and update `registry.rs`

#### Files to Modify:
- `src/backends/generic.rs` - Add fallback logic
- `src/backends/config.rs` - Add fallback field
- `src/packages/registry.rs` - Simplify to only use GenericManager
- `src/packages/mod.rs` - Remove old modules

#### Files to Delete:
- `src/packages/aur.rs`
- `src/packages/flatpak.rs`
- `src/packages/soar.rs`
- `src/packages/npm.rs`
- `src/packages/yarn.rs`
- `src/packages/pnpm.rs`
- `src/packages/bun.rs`
- `src/packages/cargo.rs`
- `src/packages/brew.rs`

---

### Phase 4: Init Command & Backend Generation
**Duration:** 3-4 days  
**Branch:** `refactor/v0.6-generic-core/phase4-init`

#### Tasks:
1. **`declarch init --backend <name>`**
   - Create `backends/<name>.kdl` with template
   - Auto-add import to `backends.kdl`
   - If `backends.kdl` doesn't exist, create it
   - If no `backends.kdl` import in `declarch.kdl`, add it

2. **Default backend templates**
   - `paru.kdl` template with fallback to pacman
   - `npm.kdl` template
   - `cargo.kdl` template
   - etc.

3. **`declarch init` (default)**
   - Create `declarch.kdl` with `import "backends.kdl"`
   - Create empty `backends.kdl`
   - Create `backends/` directory
   - Do NOT auto-populate backends (user must `init --backend`)

4. **Backend template library**
   - Store templates in code or external files
   - Easy to extend

#### Files to Modify:
- `src/commands/init.rs` - Major rewrite
- Create `src/backends/templates.rs` - Template definitions

#### Example Flow:
```bash
# User baru
$ declarch init
Created ~/.config/declarch/declarch.kdl
Created ~/.config/declarch/backends.kdl
Created ~/.config/declarch/backends/

# Add backend
$ declarch init --backend paru
Created ~/.config/declarch/backends/paru.kdl
Updated ~/.config/declarch/backends.kdl

# Check declarch.kdl
$ cat ~/.config/declarch/declarch.kdl
import "backends.kdl"

pkg {
  // Add packages here
}
```

---

### Phase 5: Commands Update
**Duration:** 4-5 days  
**Branch:** `refactor/v0.6-generic-core/phase5-commands`

#### Tasks:
1. **Update all commands to use new syntax**
   - `declarch install pkg:paru` → install to `pkg { paru { } }`
   - `declarch list` → read from new structure
   - `declarch sync` → use new resolver
   - etc.

2. **Backend detection**
   - From `backends.kdl` imports
   - Dynamic loading
   - No hardcoded list

3. **Error messages**
   - "Backend 'paru' not found. Did you forget to import?"
   - "No backends configured. Run 'declarch init --backend <name>'"

#### Files to Modify:
- `src/commands/install.rs`
- `src/commands/sync/mod.rs`
- `src/commands/list.rs`
- `src/commands/check.rs`
- `src/commands/search.rs`
- `src/commands/switch.rs`

---

### Phase 6: Testing & Documentation
**Duration:** 5-7 days  
**Branch:** `refactor/v0.6-generic-core/phase6-docs`

#### Tasks:
1. **Unit tests**
   - New parser tests
   - GenericManager fallback tests
   - Backend loading tests

2. **Integration tests**
   - End-to-end init flow
   - Install/sync/remove cycle
   - Fallback behavior

3. **Documentation**
   - Update README.md
   - New configuration guide
   - Migration guide from v0.5
   - Backend authoring guide

4. **Example configs**
   - `examples/minimal.kdl`
   - `examples/full.kdl`
   - `examples/backends/paru.kdl`
   - etc.

---

## 📊 Migration Path

### For Existing Users (v0.5 → v0.6)

```bash
# Migration tool (optional)
$ declarch migrate
Detecting v0.5 config...
Converting packages { } to pkg { }...
Creating backend definitions...
Done! Backup at ~/.config/declarch/backup-v0.5/
```

### Manual Migration

**Old (v0.5):**
```kdl
packages {
  vim
  npm:typescript
}
```

**New (v0.6):**
```kdl
import "backends.kdl"

pkg {
  paru {
    vim
  }
  npm {
    typescript
  }
}
```

---

## 🚦 Success Criteria

1. **No hardcoded backends** - `grep -r "Backend::" src/` returns only `Backend::Generic`
2. **All tests pass** - `cargo test` green
3. **Clippy clean** - `cargo clippy` zero warnings
4. **Binary size** - Reduced by ~30% (removed custom impls)
5. **Works on fresh install** - 
   ```bash
   declarch init
   declarch init --backend paru
   declarch sync
   ```

---

## ⚠️ Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Breaking changes | High | Clean break, major version bump (v0.6), good migration docs |
| Complex fallback logic | Medium | Keep fallback simple (single string), test thoroughly |
| Performance regression | Low | Benchmark before/after, optimize hot paths |
| User confusion | Medium | Clear error messages, helpful suggestions |

---

## 🎯 Checklist per Phase

- [ ] Phase 1: Type system refactored, tests passing
- [ ] Phase 2: Parser handles new syntax, deprecation warnings working
- [ ] Phase 3: GenericManager enhanced, old code deleted
- [ ] Phase 4: Init command working, templates created
- [ ] Phase 5: All commands updated
- [ ] Phase 6: Tests comprehensive, docs complete
- [ ] Final: Merge to main, tag v0.6.0

---

**Ready to start Phase 1?**
