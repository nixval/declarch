# ADR-003: Generic-First Backend Implementation Pattern

**Status:** Accepted  
**Date:** 2026-02-02  
**Author:** @nixval (via kimioc)  
**Related:** ADR-001, ADR-002, PHASE2-REFACTOR-PLAN.md

---

## Context

Setelah decision untuk unify backend implementations (ADR-001) dan refactor RawConfig (ADR-002), kita perlu mendefinisikan **when and how** untuk use generic vs custom implementations.

Current situation:
- 9 custom implementations (AUR, Flatpak, Soar, npm, yarn, pnpm, bun, cargo, brew)
- 1 generic implementation (pip only)
- Tidak ada clear criteria untuk memilih antara keduanya

---

## Decision

Adopt **Generic-First Implementation Pattern**:

```
┌─────────────────────────────────────────────────────────────┐
│                    DECISION TREE                            │
└─────────────────────────────────────────────────────────────┘

New Backend Requested
         │
         ▼
┌──────────────────────┐
│ Can it be described  │
│ by BackendConfig?    │
│ (commands, parsers)  │
└──────────┬───────────┘
           │
     ┌─────┴──────┐
     │ YES        │ NO
     ▼            ▼
┌──────────┐  ┌──────────────────┐
│ GENERIC  │  │ Custom Analysis  │
│ Start    │  └────────┬─────────┘
│ with     │           │
│ Generic  │    ┌──────┴──────┐
│ Manager  │    │ Simple      │ Complex
│          │    │ enough for  │ (state,
│          │    │ full custom │ remotes, etc)
│          │    ▼             ▼
│          │ ┌─────────┐  ┌──────────┐
│          │ │ GENERIC │  │ CUSTOM   │
│          │ │ +       │  │          │
│          │ │ Custom  │  │          │
│          │ │ Hooks   │  │          │
│          │ └─────────┘  └──────────┘
└──────────┘
```

### Decision Criteria

#### Use **GenericManager** (100% config-driven) when:

1. **Standard Command Pattern**
   - Install: `cmd install [options] <packages>`
   - Remove: `cmd uninstall/remove [options] <packages>`
   - List: `cmd list` dengan parseable output

2. **Parseable Output**
   - JSON (with optional path: e.g., `dependencies`)
   - Whitespace-separated (e.g., `brew list`)
   - Tab-separated (e.g., `flatpak list`)
   - Regex-extractable (e.g., `cargo install --list`)

3. **No Special State Management**
   - No complex initialization
   - No special binary detection (e.g., paru vs yay)
   - No cross-backend delegation (except via config)

4. **No Backend-Specific Features**
   - No remotes/repositories management
   - No variant handling (-bin, -git)
   - No auto-installation capability

**Examples:** npm, yarn, pnpm, bun, pip, cargo, brew, gem, cabal

#### Use **Custom Implementation** (Rust code) when:

1. **Complex State Management**
   - Binary detection (e.g., paru vs yay for AUR)
   - Helper selection logic
   - Cross-backend coordination

2. **Non-Standard Output**
   - ANSI codes yang harus di-strip (Soar)
   - Complex table formats
   - Multiple output formats depending on flags

3. **Special Initialization**
   - Auto-installation (Soar installs itself)
   - Remote management (Flatpak remotes)
   - First-run setup

4. **Backend-Specific Features**
   - Package variants (AUR: -bin, -git)
   - Repository management
   - Special flags atau options

**Examples:** AUR, Flatpak, Soar, nix (future), snap (future)

#### Use **Hybrid** (Generic + Custom Hooks) when:

1. **Mostly standard** tapi butuh custom behavior untuk specific operations
2. **Can extend GenericManager** dengan override methods

```rust
pub struct CustomManager {
    generic: GenericManager,
    custom_field: String,
}

impl PackageManager for CustomManager {
    // Delegate standard operations
    fn install(&self, packages: &[String]) -> Result<()> {
        self.generic.install(packages)
    }
    
    // Override untuk custom behavior
    fn list_installed(&self) -> Result<HashMap<String, PackageMetadata>> {
        // Custom implementation
    }
}
```

---

## GenericManager Capabilities

### Current

- [x] Install command execution
- [x] Remove command execution
- [x] List command dengan multiple parsers (JSON, whitespace, TSV, regex)
- [x] Noconfirm flag support
- [x] Sudo detection
- [x] Environment variable injection
- [x] Binary fallback (e.g., pip3 → pip)

### To Add (Phase 2B)

- [ ] **Search support**: Configurable search command dan parser
- [ ] **Delegate listing**: Use another backend untuk listing (yarn → npm)
- [ ] **Line regex**: Parse output line-by-line dengan regex
- [ ] **Pre/post hooks**: Execute custom commands before/after operations
- [ ] **Multi-command**: Split single operation into multiple commands

### BackendConfig Extensions

```rust
pub struct BackendConfig {
    // ... existing fields ...
    
    // NEW: Search configuration
    pub search: Option<SearchConfig>,
    
    // NEW: Delegate listing to another backend
    pub delegate_list_to: Option<Backend>,
    
    // NEW: Hooks
    pub pre_install: Option<String>,
    pub post_install: Option<String>,
    pub pre_remove: Option<String>,
    pub post_remove: Option<String>,
}

pub struct SearchConfig {
    pub cmd: String,
    pub format: OutputFormat,
    pub json_path: Option<String>,
    pub name_key: Option<String>,
    pub version_key: Option<String>,
    pub desc_key: Option<String>,
}
```

---

## Implementation Examples

### Example 1: npm (Generic)

```rust
// src/backends/registry.rs
BackendConfig {
    name: "npm".to_string(),
    binary: BinarySpecifier::Single("npm".to_string()),
    list_cmd: "npm list -g --json --depth=0".to_string(),
    install_cmd: "npm install -g {packages}".to_string(),
    remove_cmd: "npm uninstall -g {packages}".to_string(),
    list_format: OutputFormat::Json,
    list_json_path: Some("dependencies".to_string()),
    // npm JSON: { "dependencies": { "package": { "version": "1.0.0" } } }
    // name is the key, version is nested
    list_name_key: None,  // Names are JSON keys
    list_version_key: Some("version".to_string()),
    
    search: Some(SearchConfig {
        cmd: "npm search {query} --json".to_string(),
        format: OutputFormat::Json,
        name_key: Some("name".to_string()),
        desc_key: Some("description".to_string()),
        ..Default::default()
    }),
    
    noconfirm_flag: None,  // npm doesn't need confirmation
    needs_sudo: false,
}
```

### Example 2: yarn (Generic dengan Delegation)

```rust
BackendConfig {
    name: "yarn".to_string(),
    binary: BinarySpecifier::Single("yarn".to_string()),
    // yarn global list is unreliable, delegate to npm
    list_cmd: "npm list -g --json --depth=0".to_string(),
    install_cmd: "yarn global add {packages}".to_string(),
    remove_cmd: "yarn global remove {packages}".to_string(),
    list_format: OutputFormat::Json,
    list_json_path: Some("dependencies".to_string()),
    
    // NEW: Delegate listing to npm (optional)
    delegate_list_to: Some(Backend::Npm),
    
    search: Some(SearchConfig {
        cmd: "yarn search {query}".to_string(),
        format: OutputFormat::Custom,  // Need custom parsing
        ..Default::default()
    }),
}
```

### Example 3: cargo (Generic dengan Regex)

```rust
BackendConfig {
    name: "cargo".to_string(),
    binary: BinarySpecifier::Single("cargo".to_string()),
    list_cmd: "cargo install --list".to_string(),
    install_cmd: "cargo install {packages}".to_string(),
    remove_cmd: "cargo uninstall {packages}".to_string(),
    
    // cargo output: "package_name v0.1.0:"
    list_format: OutputFormat::RegexPerLine,
    list_regex: Some(r"^(\S+)\s+v([^:]+):"),
    list_regex_name_group: Some(1),
    list_regex_version_group: Some(2),
    
    search: Some(SearchConfig {
        cmd: "cargo search {query} --limit 20".to_string(),
        format: OutputFormat::RegexPerLine,
        // cargo search: "package_name = \"0.1.0\"    Description..."
        search_regex: Some(r"^(\S+)\s+=\s+\"([^\"]+)\"\s+(.*)$"),
        search_regex_name_group: Some(1),
        search_regex_version_group: Some(2),
        search_regex_desc_group: Some(3),
    }),
}
```

### Example 4: AUR (Custom dengan Generic Extension)

```rust
// src/packages/aur.rs
pub struct AurManager {
    inner: GenericManager,
    helper: String,  // paru atau yay
}

impl AurManager {
    pub fn new(helper: String, noconfirm: bool) -> Self {
        let config = BackendConfig {
            name: "aur".to_string(),
            binary: BinarySpecifier::Single(helper.clone()),
            list_cmd: format!("{} -Qq", helper),
            install_cmd: format!("{} -S {{packages}}", helper),
            remove_cmd: format!("{} -R {{packages}}", helper),
            list_format: OutputFormat::SplitWhitespace,
            noconfirm_flag: Some("--noconfirm".to_string()),
            needs_sudo: true,
            ..Default::default()
        };
        
        Self {
            inner: GenericManager::from_config(config, Backend::Aur, noconfirm),
            helper,
        }
    }
}

impl PackageManager for AurManager {
    fn backend_type(&self) -> Backend { Backend::Aur }
    
    fn is_available(&self) -> bool {
        which::which(&self.helper).is_ok()
    }
    
    fn install(&self, packages: &[String]) -> Result<()> {
        // Delegate to generic untuk standard install
        self.inner.install(packages)
    }
    
    fn list_installed(&self) -> Result<HashMap<String, PackageMetadata>> {
        // Delegate to generic, tapi dengan AUR-specific metadata extraction
        let mut packages = self.inner.list_installed()?;
        
        // AUR-specific: identify variants (-bin, -git)
        for (name, metadata) in packages.iter_mut() {
            metadata.variant = detect_aur_variant(name);
        }
        
        Ok(packages)
    }
    
    // Custom methods untuk AUR-specific features
    pub fn get_helper(&self) -> &str { &self.helper }
    pub fn supports_search(&self) -> bool { true }
}
```

---

## Consequences

### Positive

1. **Clear Guidelines**
   - Contributors know exactly when to use generic vs custom
   - Reduces decision fatigue dan subjective choices

2. **Faster Backend Addition**
   - Simple backends: 30 menit (config only)
   - Complex backends: 2-4 jam (custom implementation)

3. **Consistent Behavior**
   - Generic backends behave consistently (same error handling, same patterns)
   - Custom backends clearly marked dan documented

4. **Easier Maintenance**
   - Fix di GenericManager fixes all generic backends
   - Custom implementations isolated dan minimal

### Negative

1. **GenericManager Complexity**
   - Must support many edge cases (regex, delegation, hooks)
   - Risk of over-engineering

2. **Learning Curve**
   - Contributors must understand BackendConfig schema
   - Must know when to "escalate" ke custom implementation

3. **Migration Cost**
   - Existing custom implementations must be evaluated ulang
   - Some might need partial rewrite

### Neutral

1. **Hybrid Implementations**
   - Adds complexity tapi provides flexibility
   - Document carefully untuk avoid confusion

---

## Guidelines for Contributors

### Adding a New Backend: Checklist

```markdown
## Backend Addition Checklist

### 1. Analysis
- [ ] Can it use standard install/remove/list commands?
- [ ] Is output parseable dengan existing parsers (JSON, whitespace, TSV, regex)?
- [ ] Does it need special initialization atau state management?
- [ ] Does it need backend-specific features (remotes, variants)?

### 2. Decision
- [ ] If YES to questions 1-2 dan NO to 3-4 → **Generic**
- [ ] If NO to any question → **Custom** atau **Hybrid**

### 3. Implementation
#### Generic:
- [ ] Add BackendConfig ke `backends/registry.rs`
- [ ] Add Backend variant ke `core/types.rs`
- [ ] Register parser di `kdl_modules/registry.rs`
- [ ] Test dengan existing test suite

#### Custom/Hybrid:
- [ ] Create `packages/<backend>.rs`
- [ ] Implement `PackageManager` trait
- [ ] Add Backend variant ke `core/types.rs`
- [ ] Register di `packages/registry.rs`
- [ ] Write unit tests
- [ ] Document custom features

### 4. Validation
- [ ] All tests pass
- [ ] Manual testing: install, remove, list, search
- [ ] Error handling tested
- [ ] Documentation updated
```

---

## References

- [Phase 2 Refactoring Plan](/docs/plans/PHASE2-REFACTOR-PLAN.md)
- [Backend Unification ADR](/docs/architecture/adr/ADR-001-backend-unification.md)
- [RawConfig HashMap ADR](/docs/architecture/adr/ADR-002-rawconfig-hashmap.md)
- [Generic Manager](/src/backends/generic.rs)
- [Backend Config](/src/backends/config.rs)
- [Package Manager Trait](/src/packages/traits.rs)

---

## Success Metrics

1. **Decision Clarity**: 100% of new backend PRs follow this pattern without debate
2. **Implementation Time**:
   - Simple backends: < 1 hour
   - Complex backends: < 4 hours
3. **Code Consistency**: All generic backends share 95%+ code paths
4. **Test Coverage**: All backends have > 80% test coverage

---

## Notes

- **Document Decisions**: When choosing custom over generic, document why di ADR comments
- **Re-evaluate**: Periodically check if custom implementations can be migrated ke generic
- **Extend GenericManager**: When multiple custom implementations share pattern, consider adding ke GenericManager
