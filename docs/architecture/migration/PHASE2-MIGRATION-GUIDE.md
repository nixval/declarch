# Declarch Phase 2 Migration Guide

**Version:** 1.0  
**Date:** 2026-02-02  
**Branch:** architect-kimioc  
**Target:** Phase 2 Backend Unification & RawConfig Refactoring

---

## Table of Contents

1. [Overview](#overview)
2. [Migration Phases](#migration-phases)
3. [Code Changes by Module](#code-changes-by-module)
4. [Testing Strategy](#testing-strategy)
5. [Rollback Procedures](#rollback-procedures)
6. [Troubleshooting](#troubleshooting)
7. [FAQ](#faq)

---

## Overview

### What is Being Changed?

Phase 2 refactoring addresses fundamental architectural issues:

1. **Backend Duplication** (`packages/` vs `backends/`)
   - 9 custom implementations → 4 (AUR, Flatpak, Soar + pip yang sudah generic)
   - Simple backends (npm, yarn, pnpm, bun, cargo, brew) → GenericManager

2. **RawConfig Proliferation**
   - 25+ fields dengan per-backend vectors → 12 fields dengan single HashMap
   - 10+ individual parsers → 1 generic parser

3. **Backend Addition Complexity**
   - 12+ files modified → 2-3 files

### Impact Assessment

| Aspect | Impact | Notes |
|--------|--------|-------|
| **User-Facing Config** | ✅ No Change | KDL format tetap sama |
| **CLI Commands** | ✅ No Change | All commands work identically |
| **State File** | ✅ No Change | state.json format tetap sama |
| **Public API** | ⚠️ Minor | Library users may need updates |
| **Internal API** | 🔄 Major | RawConfig field access berubah |

---

## Migration Phases

### Phase 2A: RawConfig HashMap (Days 1-5)

**Goal:** Refactor RawConfig untuk use HashMap<Backend, Vec<PackageEntry>>

#### Step 1: Update RawConfig (Day 1)

**Before:**
```rust
pub struct RawConfig {
    pub packages: Vec<PackageEntry>,           // AUR
    pub npm_packages: Vec<PackageEntry>,       // npm
    pub yarn_packages: Vec<PackageEntry>,     // yarn
    // ... 7 more ...
}
```

**After:**
```rust
pub struct RawConfig {
    pub packages: HashMap<Backend, Vec<PackageEntry>>,  // All backends
}

impl RawConfig {
    pub fn aur_packages(&self) -> &Vec<PackageEntry> {
        self.packages.get(&Backend::Aur).unwrap_or(&EMPTY_VEC)
    }
    // ... accessor untuk all backends
}
```

**Migration Commands:**
```bash
# Search for all direct field access
grep -r "raw\.packages" src/ --include="*.rs" | grep -v test
grep -r "raw\..*_packages" src/ --include="*.rs" | grep -v test

# Update dengan accessor methods
# sed -i 's/raw\.packages/raw\.aur_packages()/g' src/commands/check.rs
# (Manual review recommended)
```

#### Step 2: Create Generic Parser (Day 2)

**Before:** 10+ parser files di `src/config/kdl_modules/parsers/`

**After:** Single generic parser

**Migration:**
```bash
# Backup old parsers
mkdir -p src/config/kdl_modules/parsers/deprecated
git mv src/config/kdl_modules/parsers/npm.rs src/config/kdl_modules/parsers/deprecated/
# ... etc untuk all simple backends

# Create generic parser
touch src/config/kdl_modules/parsers/generic.rs
```

#### Step 3: Update All Access Sites (Days 3-4)

**Files to Update:**

1. `src/config/loader.rs`
   ```rust
   // Before
   for pkg in &raw.packages { process(pkg, Backend::Aur); }
   for pkg in &raw.npm_packages { process(pkg, Backend::Npm); }
   
   // After
   for (backend, packages) in &raw.packages {
       for pkg in packages { process(pkg, backend); }
   }
   ```

2. `src/commands/check.rs`
   ```rust
   // Before
   backend_packages.extend(raw.npm_packages.clone());
   
   // After
   if let Some(packages) = raw.packages.get(&Backend::Npm) {
       backend_packages.extend(packages.clone());
   }
   ```

3. `src/commands/install.rs` (similar pattern)
4. `src/core/matcher.rs` (similar pattern)
5. `src/config/kdl_modules/registry.rs` (inline prefix parsing)

#### Step 4: Testing (Day 5)

```bash
# Run all tests
cargo test

# Run KDL tests specifically
cargo test kdl

# Test config loading
cargo test config

# Manual test
cargo run -- info
```

---

### Phase 2B: Backend Unification (Days 6-14)

**Goal:** Migrate simple backends ke GenericManager

#### Step 1: Enhance GenericManager (Days 6-7)

**Add to `src/backends/generic.rs`:**

1. Search support
2. Delegate listing (yarn → npm)
3. Line-by-line regex parsing

**Testing:**
```bash
cargo test backends::generic
```

#### Step 2: Create BackendConfigs (Days 8-9)

**Add to `src/backends/registry.rs`:**

```rust
pub fn get_builtin_backends() -> HashMap<String, BackendConfig> {
    let mut backends = HashMap::new();
    
    // npm
    backends.insert("npm".to_string(), BackendConfig {
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
        // ... etc
    });
    
    // yarn, pnpm, bun, cargo, brew (similar)
    
    backends
}
```

#### Step 3: Update BackendRegistry (Day 10)

**Modify `src/packages/registry.rs`:**

```rust
impl BackendRegistry {
    pub fn register_defaults(&mut self) {
        // Complex backends - tetap custom
        self.register(Backend::Aur, |config, noconfirm| {
            Ok(Box::new(AurManager::new(config.aur_helper.clone(), noconfirm)))
        });
        self.register(Backend::Flatpak, |_config, noconfirm| {
            Ok(Box::new(FlatpakManager::new(noconfirm)))
        });
        self.register(Backend::Soar, |_config, noconfirm| {
            Ok(Box::new(SoarManager::new(noconfirm)))
        });
        
        // Simple backends - now all generic!
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

#### Step 4: Remove Old Implementations (Day 11)

```bash
# Remove old custom implementations
git rm src/packages/npm.rs
git rm src/packages/yarn.rs
git rm src/packages/pnpm.rs
git rm src/packages/bun.rs
git rm src/packages/cargo.rs
git rm src/packages/brew.rs

# Update mod.rs
cat > src/packages/mod.rs << 'EOF'
pub mod aur;
pub mod flatpak;
pub mod registry;
pub mod soar;
pub mod traits;

pub use registry::{BackendRegistry, create_manager, get_registry};
pub use traits::PackageManager;
EOF
```

#### Step 5: Testing (Days 12-14)

```bash
# Full test suite
cargo test

# Backend-specific tests
for backend in npm yarn pnpm bun cargo brew; do
    echo "Testing $backend..."
    cargo test $backend
done

# Integration testing
cargo test integration

# Build release
cargo build --release

# Manual testing
cargo run -- sync --dry-run
cargo run -- info
cargo run -- list
```

---

## Code Changes by Module

### 1. Config Module (`src/config/`)

#### Changes:
- `kdl_modules/types.rs`: Refactor RawConfig struct
- `kdl_modules/registry.rs`: Update untuk use HashMap
- `kdl_modules/parser.rs`: Update untuk HashMap access
- `loader.rs`: Unified backend iteration

#### Migration Pattern:
```rust
// Field access migration
raw.npm_packages → raw.npm_packages()  // accessor
raw.packages → raw.aur_packages()      // accessor (AUR was default)

// Iteration migration
for pkg in &raw.npm_packages { ... } →
for pkg in raw.npm_packages() { ... }

// Insertion migration
raw.npm_packages.push(entry) →
raw.packages.entry(Backend::Npm).or_default().push(entry)
```

### 2. Commands Module (`src/commands/`)

#### Changes:
- `check.rs`: Update backend parsing
- `install.rs`: Update backend parsing
- `sync/planner.rs`: Update package filtering

#### Migration Pattern:
```rust
// Backend iteration
for backend in [Backend::Aur, Backend::Npm, ...] { ... } →
for backend in raw.packages.keys() { ... }

// Package access by backend
let packages = match backend {
    Backend::Npm => &raw.npm_packages,
    ...
} →
let packages = raw.packages.get(backend).unwrap_or(&EMPTY_VEC)
```

### 3. Core Module (`src/core/`)

#### Changes:
- `types.rs`: Add Backend accessor helpers
- `matcher.rs`: Update package lookup

### 4. Packages Module (`src/packages/`)

#### Changes:
- `registry.rs`: Unified backend registration
- `mod.rs`: Remove old module exports
- Delete 6 implementation files

### 5. Backends Module (`src/backends/`)

#### Changes:
- `registry.rs`: Add BackendConfigs untuk all backends
- `generic.rs`: Add search support

---

## Testing Strategy

### Pre-Migration Baseline

```bash
# Record current state
git log --oneline -10 > migration/baseline-commits.txt
cargo test 2>&1 | tee migration/baseline-tests.txt
cargo build --release 2>&1 | tee migration/baseline-build.txt
wc -l src/**/*.rs > migration/baseline-lines.txt
```

### Per-Phase Testing

#### Phase 2A Tests:
```bash
# Unit tests
cargo test config::kdl_modules
cargo test raw_config

# Integration tests
cargo test config_loading

# Validation
./target/release/declarch info
./target/release/declarch check
```

#### Phase 2B Tests:
```bash
# Backend unit tests
cargo test packages::registry
cargo test backends::generic

# Integration tests untuk setiap backend
for backend in npm yarn pnpm bun cargo brew pip; do
    cargo test $backend
done

# End-to-end tests
cargo test integration

# Manual tests
./target/release/declarch sync --dry-run
./target/release/declarch list
./target/release/declarch info
```

### Post-Migration Validation

```bash
# Compare dengan baseline
cargo test 2>&1 | diff migration/baseline-tests.txt -
cargo build --release 2>&1 | diff migration/baseline-build.txt -
wc -l src/**/*.rs | diff migration/baseline-lines.txt -

# Performance test
time ./target/release/declarch sync --dry-run

# Functionality test
./target/release/declarch check
./target/release/declarch info doctor
```

---

## Rollback Procedures

### Phase 2A Rollback

```bash
# Revert RawConfig changes
git checkout src/config/kdl_modules/types.rs
git checkout src/config/kdl_modules/registry.rs
git checkout src/config/kdl_modules/parser.rs
git checkout src/config/loader.rs

# Restore old parsers
git checkout src/config/kdl_modules/parsers/

# Revert command changes
git checkout src/commands/check.rs
git checkout src/commands/install.rs

# Verify
cargo test
cargo build --release
```

### Phase 2B Rollback

```bash
# Revert registry changes
git checkout src/packages/registry.rs
git checkout src/backends/registry.rs

# Restore old implementations
git checkout src/packages/npm.rs
git checkout src/packages/yarn.rs
git checkout src/packages/pnpm.rs
git checkout src/packages/bun.rs
git checkout src/packages/cargo.rs
git checkout src/packages/brew.rs
git checkout src/packages/mod.rs

# Verify
cargo test
cargo build --release
```

### Full Rollback

```bash
# Reset to pre-migration state
git reset --hard architect-kimioc~1

# Or jika sudah committed
git revert --no-commit HEAD~5..HEAD
```

---

## Troubleshooting

### Common Issues

#### Issue 1: "cannot find value `npm_packages` in module `types`"

**Cause:** Code masih mengakses field lama

**Fix:**
```bash
# Find all occurrences
grep -rn "npm_packages\|yarn_packages\|pnpm_packages" src/

# Replace dengan accessor
# Manual atau use sed
sed -i 's/\.npm_packages/.npm_packages()/g' src/commands/check.rs
```

#### Issue 2: "Backend not found in HashMap"

**Cause:** Backend tidak di-initialize di RawConfig::default()

**Fix:**
```rust
impl Default for RawConfig {
    fn default() -> Self {
        let mut packages = HashMap::new();
        packages.insert(Backend::Aur, Vec::new());
        packages.insert(Backend::Npm, Vec::new());
        // ... all backends
        
        Self { packages, ... }
    }
}
```

#### Issue 3: "Backend registry panic"

**Cause:** Backend registered di registry tapi tidak ada config

**Fix:**
```rust
// Ensure all registered backends have configs
let builtin = get_builtin_backends();
assert!(builtin.contains_key("npm"));
assert!(builtin.contains_key("yarn"));
// ... etc
```

#### Issue 4: "GenericManager install failed"

**Cause:** Command template tidak sesuai

**Debug:**
```rust
// Add debug logging
println!("Install command: {}", self.config.install_cmd);
println!("Packages: {:?}", packages);
```

---

## FAQ

### Q1: Will my existing KDL configs still work?

**A:** ✅ Yes! User-facing KDL format tidak berubah. All existing configs work identically.

### Q2: Do I need to reinstall all packages?

**A:** ✅ No. State file format tetap sama. `state.json` tidak berubah.

### Q3: Will this break my scripts?

**A:** ✅ No. CLI commands dan output format tetap sama.

### Q4: What if a backend doesn't work dengan GenericManager?

**A:** We can always add custom implementation back. ADR-003 provides criteria untuk when to use custom.

### Q5: How do I add a new backend setelah refactoring?

**A:** See [Adding a New Backend](#adding-a-new-backend) section di ADR-003.

### Q6: Can I use the old custom implementations?

**A:** Yes, dengan restore files dari git history. Tapi direkomendasikan untuk use generic pattern.

### Q7: Will this improve performance?

**A:** Minimal impact. GenericManager uses same command execution. Might slightly improve due to reduced code paths.

### Q8: How do I test changes during migration?

**A:** See [Testing Strategy](#testing-strategy) section. Run `cargo test` setiap step.

### Q9: What if I find a bug setelah migration?

**A:** 
1. Check [Troubleshooting](#troubleshooting) section
2. Jika critical, use [Rollback](#rollback-procedures)
3. Report issue dengan detail

### Q10: When will this be merged ke main?

**A:** After:
1. All 159 tests pass
2. Manual testing complete
3. Performance benchmark acceptable
4. Code review approved

---

## Additional Resources

- [Phase 2 Refactoring Plan](/docs/plans/PHASE2-REFACTOR-PLAN.md)
- [ADR-001: Backend Unification](/docs/architecture/adr/ADR-001-backend-unification.md)
- [ADR-002: RawConfig HashMap](/docs/architecture/adr/ADR-002-rawconfig-hashmap.md)
- [ADR-003: Generic-First Pattern](/docs/architecture/adr/ADR-003-generic-first-pattern.md)
- [Contributing Guide](/CONTRIBUTING.md)

---

## Contact

Jika ada questions atau issues selama migration:

1. Check this guide
2. Check ADRs untuk detailed rationale
3. Open issue di repository
4. Contact: @nixval

---

**Happy Migrating! 🚀**
