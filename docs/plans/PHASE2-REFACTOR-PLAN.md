# Declarch Phase 2: Backend Unification & Architecture Refactoring

**Version:** 1.0  
**Date:** 2026-02-02  
**Author:** @nixval (via kimioc)  
**Status:** Planned  
**Branch:** architect-kimioc  
**Estimated Duration:** 14-21 days  

---

## Executive Summary

### Current State (Post-Phase 1)

Phase 1 refactoring telah berhasil:
- ✅ KDL parsing refactored (1,478 → 31 lines)
- ✅ CLI dispatcher created (main.rs: 560 → 31 lines)
- ✅ Sync modules extracted (6 sub-modules)
- ✅ 159 tests passing

### Critical Issues Identified

1. **Backend Duplication**: 9 custom implementations + 1 generic (underused)
2. **RawConfig Proliferation**: 25+ fields dengan per-backend vectors
3. **High Addition Cost**: 12+ files modified untuk add 1 backend

### Phase 2 Goals

- Unify backend implementations ke generic infrastructure
- Refactor RawConfig untuk use HashMap<Backend, Vec<PackageEntry>>
- Reduce backend addition cost dari 12+ files ke 2-3 files
- Maintain 100% backward compatibility

---

## Architecture Analysis

### Current Architecture Issues

```
┌──────────────────────────────────────────────────────────────┐
│                    CURRENT PROBLEMS                          │
├──────────────────────────────────────────────────────────────┤
│ 1. DUPLICATION                                               │
│    packages/: 9 custom implementations (~1500 lines)         │
│    backends/: Generic infrastructure, hanya 1 backend pakai  │
│                                                              │
│ 2. PROLIFERATION                                             │
│    RawConfig: 25+ fields, per-backend vectors                │
│    Adding backend: Modify struct + 10+ files                 │
│                                                              │
│ 3. INCONSISTENCY                                             │
│    npm/yarn/pnpm/bun/cargo/brew: Simple backends             │
│    Tapi semua pakai custom implementations                   │
│    Padahal bisa pakai GenericManager                         │
└──────────────────────────────────────────────────────────────┘
```

### Target Architecture

```
┌──────────────────────────────────────────────────────────────┐
│                    TARGET ARCHITECTURE                       │
├──────────────────────────────────────────────────────────────┤
│ 1. UNIFICATION                                               │
│    Complex (AUR, Flatpak, Soar): Custom implementations      │
│    Simple (npm, yarn, etc.): GenericManager                  │
│    User-defined: GenericManager dengan config                │
│                                                              │
│ 2. SIMPLIFICATION                                            │
│    RawConfig: 12 fields, single HashMap                      │
│    Adding backend: 2-3 files only                            │
│                                                              │
│ 3. CONSISTENCY                                               │
│    Generic-First pattern: Config-driven by default           │
│    Custom only untuk complex cases                           │
└──────────────────────────────────────────────────────────────┘
```

---

## Phase Breakdown

### Phase 2A: RawConfig HashMap Refactoring (Days 1-5)

**Objective:** Eliminate per-backend vectors, adopt HashMap<Backend, Vec<PackageEntry>>

#### 2A.1 Update RawConfig Struct (Day 1)

**Current:**
```rust
pub struct RawConfig {
    pub packages: Vec<PackageEntry>,           // AUR
    pub npm_packages: Vec<PackageEntry>,       // npm
    pub yarn_packages: Vec<PackageEntry>,      // yarn
    pub pnpm_packages: Vec<PackageEntry>,      // pnpm
    pub bun_packages: Vec<PackageEntry>,       // bun
    pub pip_packages: Vec<PackageEntry>,       // pip
    pub cargo_packages: Vec<PackageEntry>,     // cargo
    pub brew_packages: Vec<PackageEntry>,      // brew
    pub soar_packages: Vec<PackageEntry>,      // Soar
    pub flatpak_packages: Vec<PackageEntry>,   // Flatpak
    pub custom_packages: HashMap<String, Vec<PackageEntry>>,
    // ... 14 more fields
}
```

**Target:**
```rust
pub struct RawConfig {
    pub imports: Vec<String>,
    pub packages: HashMap<Backend, Vec<PackageEntry>>,  // All built-in
    pub custom_packages: HashMap<String, Vec<PackageEntry>>,
    pub excludes: Vec<String>,
    pub package_mappings: HashMap<String, String>,
    pub project_metadata: ProjectMetadata,
    pub conflicts: Vec<ConflictEntry>,
    pub backend_options: HashMap<String, HashMap<String, String>>,
    pub env: HashMap<String, Vec<String>>,
    pub package_sources: HashMap<String, Vec<String>>,
    pub policy: PolicyConfig,
    pub lifecycle_actions: LifecycleConfig,
}

impl RawConfig {
    // Backward compatibility accessors
    pub fn aur_packages(&self) -> &Vec<PackageEntry>;
    pub fn npm_packages(&self) -> &Vec<PackageEntry>;
    // ... etc
}
```

**Tasks:**
- [ ] Refactor struct definition (src/config/kdl_modules/types.rs)
- [ ] Implement Default untuk initialize all backends
- [ ] Add accessor methods untuk backward compatibility
- [ ] Add `all_packages()` iterator

**Success Criteria:**
- RawConfig field count: 12 (was 25+)
- Backward compatibility maintained via accessors

#### 2A.2 Create Generic Backend Parser (Day 2)

**Current:** 10+ individual parser files

**Target:** Single generic parser

**Tasks:**
- [ ] Create `src/config/kdl_modules/parsers/generic.rs`
- [ ] Implement `GenericBackendParser` struct
- [ ] Implement `BackendParser` trait
- [ ] Support backend aliases (e.g., "aur", "repo" for AUR)

**Code:**
```rust
pub struct GenericBackendParser {
    backend: Backend,
    aliases: Vec<String>,
}

impl BackendParser for GenericBackendParser {
    fn backend(&self) -> &str { self.backend.as_str() }
    fn matches(&self, name: &str) -> bool {
        self.aliases.contains(&name.to_string())
    }
    fn parse(&self, node: &KdlNode, config: &mut RawConfig) -> Result<()> {
        let packages = helpers::packages::extract_packages(node);
        config.packages
            .entry(self.backend.clone())
            .or_default()
            .extend(packages);
        Ok(())
    }
}
```

**Success Criteria:**
- Parser implementations: 1 generic (was 10+)
- All existing tests pass

#### 2A.3 Update Parser Registry (Day 2-3)

**Current:** Vec<Box<dyn BackendParser>>

**Target:** HashMap<Backend, Box<dyn BackendParser>>

**Tasks:**
- [ ] Update `BackendParserRegistry` untuk use HashMap
- [ ] Register all backends dengan generic parser
- [ ] Remove old parser files

**Code:**
```rust
impl BackendParserRegistry {
    pub fn new() -> Self {
        let mut parsers = HashMap::new();
        parsers.insert(Backend::Aur, Box::new(GenericBackendParser::new(
            Backend::Aur, vec!["aur", "repo"]
        )));
        parsers.insert(Backend::Npm, Box::new(GenericBackendParser::new(
            Backend::Npm, vec!["npm"]
        )));
        // ... etc untuk all backends
        Self { parsers }
    }
}
```

**Files Modified:**
- src/config/kdl_modules/registry.rs
- src/config/kdl_modules/parsers/mod.rs (remove old modules)

**Files Deleted:**
- src/config/kdl_modules/parsers/npm.rs
- src/config/kdl_modules/parsers/yarn.rs
- src/config/kdl_modules/parsers/pnpm.rs
- src/config/kdl_modules/parsers/bun.rs
- src/config/kdl_modules/parsers/cargo.rs
- src/config/kdl_modules/parsers/brew.rs
- src/config/kdl_modules/parsers/pip.rs
- src/config/kdl_modules/parsers/soar.rs (keep? complex parsing)
- src/config/kdl_modules/parsers/flatpak.rs (keep? complex parsing)
- src/config/kdl_modules/parsers/aur.rs (keep? complex parsing)

#### 2A.4 Update All Access Sites (Days 3-4)

**Files to Update:**

1. `src/config/loader.rs`
   - Update backend iteration
   - Use unified `all_packages()` iterator

2. `src/commands/check.rs`
   - Update `parse_backend()` untuk use HashMap
   - Update package access patterns

3. `src/commands/install.rs`
   - Similar updates seperti check.rs

4. `src/core/matcher.rs`
   - Update package lookup
   - Use HashMap.get() instead of field access

5. `src/config/kdl_modules/registry.rs`
   - Update inline prefix parsing
   - Route ke HashMap instead of individual vectors

**Migration Pattern:**
```rust
// Before
for pkg in &raw.packages { ... }
for pkg in &raw.npm_packages { ... }

// After
for (backend, packages) in &raw.packages {
    for pkg in packages { ... }
}

// Or dengan accessor
for pkg in raw.aur_packages() { ... }
```

#### 2A.5 Testing (Day 5)

**Tasks:**
- [ ] Run all 159 unit tests
- [ ] Add tests untuk accessor methods
- [ ] Test backward compatibility
- [ ] Performance benchmark (config loading)

**Commands:**
```bash
cargo test
cargo test config
cargo test kdl
cargo test --lib
```

**Success Criteria:**
- All tests pass
- No performance regression
- Backward compatibility verified

---

### Phase 2B: Backend Unification (Days 6-14)

**Objective:** Migrate simple backends ke GenericManager

#### 2B.1 Enhance GenericManager (Days 6-7)

**Current:** Basic install/remove/list

**Target:** Full-featured dengan search, delegation, regex

**Tasks:**

1. **Add Search Support**
   ```rust
   impl GenericManager {
       pub fn with_search(mut self, config: SearchConfig) -> Self {
           self.search_config = Some(config);
           self
       }
   }
   ```

2. **Add Delegate Listing**
   ```rust
   pub struct BackendConfig {
       // ... existing fields ...
       pub delegate_list_to: Option<Backend>,  // NEW
   }
   ```

3. **Add Line-by-Line Regex Parsing**
   ```rust
   pub enum OutputFormat {
       Json,
       SplitWhitespace,
       TabSeparated,
       RegexPerLine,  // NEW
   }
   ```

4. **Add Pre/Post Hooks**
   ```rust
   pub struct BackendConfig {
       pub pre_install: Option<String>,
       pub post_install: Option<String>,
       pub pre_remove: Option<String>,
       pub post_remove: Option<String>,
   }
   ```

**Files Modified:**
- src/backends/generic.rs
- src/backends/config.rs

#### 2B.2 Create BackendConfigs (Days 8-9)

**Tasks:** Create BackendConfig untuk setiap simple backend

1. **npm**
   ```rust
   BackendConfig {
       name: "npm".to_string(),
       binary: BinarySpecifier::Single("npm".to_string()),
       list_cmd: "npm list -g --json --depth=0".to_string(),
       install_cmd: "npm install -g {packages}".to_string(),
       remove_cmd: "npm uninstall -g {packages}".to_string(),
       list_format: OutputFormat::Json,
       list_json_path: Some("dependencies".to_string()),
       list_version_key: Some("version".to_string()),
       search_cmd: Some("npm search {query} --json".to_string()),
       search_format: Some(OutputFormat::Json),
       noconfirm_flag: None,
       needs_sudo: false,
   }
   ```

2. **yarn** (dengan npm delegation untuk list)
   ```rust
   BackendConfig {
       name: "yarn".to_string(),
       binary: BinarySpecifier::Single("yarn".to_string()),
       list_cmd: "npm list -g --json --depth=0".to_string(),  // Delegate
       install_cmd: "yarn global add {packages}".to_string(),
       remove_cmd: "yarn global remove {packages}".to_string(),
       list_format: OutputFormat::Json,
       list_json_path: Some("dependencies".to_string()),
       delegate_list_to: Some(Backend::Npm),  // NEW
       search_cmd: Some("yarn search {query}".to_string()),
       search_format: Some(OutputFormat::Custom),  // Need custom parser
       noconfirm_flag: None,
       needs_sudo: false,
   }
   ```

3. **pnpm**, **bun**, **cargo**, **brew** (similar patterns)

**Files Modified:**
- src/backends/registry.rs (add get_builtin_backends())

#### 2B.3 Update BackendRegistry (Day 10)

**Tasks:**
- [ ] Keep custom implementations untuk AUR, Flatpak, Soar
- [ ] Migrate npm, yarn, pnpm, bun, cargo, brew ke GenericManager
- [ ] Update register_defaults()

**Code:**
```rust
impl BackendRegistry {
    pub fn register_defaults(&mut self) {
        // Complex backends - custom
        self.register(Backend::Aur, |config, noconfirm| {
            Ok(Box::new(AurManager::new(config.aur_helper.clone(), noconfirm)))
        });
        self.register(Backend::Flatpak, |_config, noconfirm| {
            Ok(Box::new(FlatpakManager::new(noconfirm)))
        });
        self.register(Backend::Soar, |_config, noconfirm| {
            Ok(Box::new(SoarManager::new(noconfirm)))
        });
        
        // Simple backends - generic
        let builtin = get_builtin_backends();
        for (name, config) in &builtin {
            if let Ok(backend) = Backend::from_str(name) {
                let config = config.clone();
                self.register(backend, move |_global_config, noconfirm| {
                    Ok(Box::new(GenericManager::from_config(
                        config.clone(),
                        backend.clone(),
                        noconfirm,
                    )))
                });
            }
        }
    }
}
```

**Files Modified:**
- src/packages/registry.rs

#### 2B.4 Remove Old Implementations (Day 11)

**Files to Delete:**
- src/packages/npm.rs
- src/packages/yarn.rs
- src/packages/pnpm.rs
- src/packages/bun.rs
- src/packages/cargo.rs
- src/packages/brew.rs

**Files to Update:**
- src/packages/mod.rs (remove exports)

**Net Reduction:** -600 lines

#### 2B.5 Testing (Days 12-14)

**Tasks:**
- [ ] Unit tests untuk setiap migrated backend
- [ ] Integration tests
- [ ] Manual testing
- [ ] Performance benchmark

**Test Matrix:**

| Backend | Install | Remove | List | Search | Status |
|---------|---------|--------|------|--------|--------|
| npm | ☐ | ☐ | ☐ | ☐ | Pending |
| yarn | ☐ | ☐ | ☐ | ☐ | Pending |
| pnpm | ☐ | ☐ | ☐ | ☐ | Pending |
| bun | ☐ | ☐ | ☐ | ☐ | Pending |
| cargo | ☐ | ☐ | ☐ | ☐ | Pending |
| brew | ☐ | ☐ | ☐ | ☐ | Pending |
| AUR | ☐ | ☐ | ☐ | ☐ | Pending |
| Flatpak | ☐ | ☐ | ☐ | ☐ | Pending |
| Soar | ☐ | ☐ | ☐ | ☐ | Pending |

**Commands:**
```bash
# Full test suite
cargo test

# Backend-specific tests
for backend in npm yarn pnpm bun cargo brew aur flatpak soar; do
    cargo test $backend
done

# Integration tests
cargo test integration

# Manual tests
./target/release/declarch sync --dry-run
./target/release/declarch list
./target/release/declarch info
./target/release/declarch check
```

---

### Phase 2C: BackendConfig Simplification (Optional, Days 15-17)

**Objective:** Reduce BackendConfig complexity dengan nested structs

**Current:** 30+ fields (list_*, search_*)

**Target:** ~15 fields dengan nested ParseConfig

**Code:**
```rust
pub struct ParseConfig {
    pub format: OutputFormat,
    pub json_path: Option<String>,
    pub name_key: Option<String>,
    pub version_key: Option<String>,
    pub desc_key: Option<String>,
    pub column_indices: Option<(usize, Option<usize>)>,
    pub regex: Option<RegexConfig>,
}

pub struct BackendConfig {
    pub name: String,
    pub binary: BinarySpecifier,
    pub list_cmd: String,
    pub install_cmd: String,
    pub remove_cmd: String,
    pub query_cmd: Option<String>,
    pub list: ParseConfig,           // Nested
    pub search: Option<ParseConfig>, // Nested
    pub noconfirm_flag: Option<String>,
    pub needs_sudo: bool,
    pub preinstall_env: Option<HashMap<String, String>>,
}
```

**Impact:** -50% fields

---

### Phase 2D: Error Handling Unification (Optional, Days 18-21)

**Objective:** Standardize error handling across backend operations

**Current Issues:**
- `create_manager` returns `Result<..., String>`
- `DeclarchError::Other(String)` catch-all

**Target:**
- All functions return `Result<..., DeclarchError>`
- Specific error variants untuk setiap case

**Code:**
```rust
#[derive(Error, Debug)]
pub enum DeclarchError {
    #[error("Backend not available: {backend}")]
    BackendNotAvailable { backend: String },
    
    #[error("Backend factory error: {backend}: {reason}")]
    BackendFactoryError { backend: String, reason: String },
    
    // Remove DeclarchError::Other
}

pub fn create_manager(...) -> Result<Box<dyn PackageManager>, DeclarchError> {
    // ...
}
```

---

## Metrics & Success Criteria

### Code Metrics

| Metric | Before | After | Target | Delta |
|--------|--------|-------|--------|-------|
| **Total Lines** | ~14,400 | ~12,800 | < 13,000 | -1,600 (-11%) |
| **RawConfig Fields** | 25+ | 12 | ≤ 12 | -50% |
| **Parser Files** | 10+ | 1-3 | ≤ 3 | -70% |
| **Backend Impl Files** | 10 | 4 | ≤ 4 | -60% |
| **Backend Addition Cost** | 12 files | 2-3 files | ≤ 3 | -75% |

### Quality Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| **Test Pass Rate** | 100% | `cargo test` |
| **Test Coverage** | ≥ 80% | `cargo tarpaulin` |
| **Clippy Warnings** | 0 | `cargo clippy` |
| **Documentation** | 100% | `cargo doc` |

### Performance Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| **Config Loading** | ≤ 110% baseline | Time `declarch info` |
| **Sync Dry-Run** | ≤ 110% baseline | Time `declarch sync --dry-run` |
| **Binary Size** | ≤ 105% baseline | `ls -la target/release/declarch` |
| **Compile Time** | ≤ 120% baseline | `time cargo build --release` |

---

## Timeline

```
Week 1 (Days 1-7)
├── Day 1: RawConfig struct refactor
├── Day 2: Generic parser + Registry
├── Day 3-4: Update access sites
├── Day 5: Testing Phase 2A
├── Day 6-7: Enhance GenericManager

Week 2 (Days 8-14)
├── Day 8-9: Create BackendConfigs
├── Day 10: Update BackendRegistry
├── Day 11: Remove old implementations
├── Day 12-14: Testing Phase 2B

Week 3 (Days 15-21) - Optional
├── Day 15-17: Phase 2C (BackendConfig simplification)
└── Day 18-21: Phase 2D (Error handling)
```

**Total Duration:** 14 days (core) + 7 days (optional) = 14-21 days

---

## Risk Assessment

| Risk | Probability | Impact | Mitigation |
|------|------------|--------|------------|
| **Test Failures** | Medium | High | Comprehensive testing setiap phase |
| **Behavioral Changes** | Medium | High | Extensive manual testing |
| **Performance Regression** | Low | Medium | Benchmarking sebelum/after |
| **Contributor Confusion** | Low | Low | Update documentation |
| **Rollback Need** | Low | Medium | Git tags untuk setiap phase |

---

## Rollback Plan

### Phase Rollback

```bash
# Phase 2A rollback
git checkout src/config/kdl_modules/types.rs
git checkout src/config/kdl_modules/registry.rs
git checkout src/config/loader.rs
git checkout src/commands/check.rs
git checkout src/commands/install.rs

# Phase 2B rollback
git checkout src/packages/registry.rs
git checkout src/packages/mod.rs
git checkout src/backends/registry.rs
git restore src/packages/npm.rs
git restore src/packages/yarn.rs
# ... etc
```

### Full Rollback

```bash
# Reset ke pre-migration state
git reset --hard architect-kimioc~1

# Or revert commits
git revert --no-commit HEAD~10..HEAD
```

---

## Documentation

### Created Documents

1. **Architecture Decision Records (ADRs)**
   - ADR-001: Backend Unification Strategy
   - ADR-002: RawConfig HashMap Refactoring
   - ADR-003: Generic-First Backend Implementation Pattern

2. **Migration Guide**
   - PHASE2-MIGRATION-GUIDE.md

3. **Master Plan**
   - This document (PHASE2-REFACTOR-PLAN.md)

### Document Locations

```
docs/
├── architecture/
│   ├── adr/
│   │   ├── ADR-001-backend-unification.md
│   │   ├── ADR-002-rawconfig-hashmap.md
│   │   └── ADR-003-generic-first-pattern.md
│   └── migration/
│       └── PHASE2-MIGRATION-GUIDE.md
└── plans/
    └── PHASE2-REFACTOR-PLAN.md
```

---

## References

- [Phase 1 Progress](/REFACTOR_PROGRESS.md)
- [Contributing Guide](/CONTRIBUTING.md)
- [README](/README.md)
- [Architecture Analysis](#architecture-analysis)

---

## Approval

| Role | Name | Approval Date |
|------|------|---------------|
| **Author** | @nixval | 2026-02-02 |
| **Reviewer** | TBD | ☐ |
| **Approver** | TBD | ☐ |

---

## Notes

- **Breaking Changes:** None untuk end users. Internal API changes only.
- **User Impact:** Zero. KDL format dan CLI commands tetap sama.
- **Future Work:** Phase 3 bisa fokus pada features (new backends, UI improvements)
- **Maintenance:** Setelah Phase 2 complete, maintenance cost akan significantly lower

---

**Ready to Execute! 🚀**
