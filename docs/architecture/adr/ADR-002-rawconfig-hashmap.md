# ADR-002: RawConfig HashMap Refactoring

**Status:** Accepted  
**Date:** 2026-02-02  
**Author:** @nixval (via kimioc)  
**Related:** ADR-001, PHASE2-REFACTOR-PLAN.md

---

## Context

`RawConfig` di `src/config/kdl_modules/types.rs` saat ini punya **25+ fields** dengan per-backend vectors:

```rust
pub struct RawConfig {
    pub packages: Vec<PackageEntry>,           // AUR
    pub soar_packages: Vec<PackageEntry>,      // Soar
    pub flatpak_packages: Vec<PackageEntry>,   // Flatpak
    pub npm_packages: Vec<PackageEntry>,       // npm
    pub yarn_packages: Vec<PackageEntry>,      // yarn
    pub pnpm_packages: Vec<PackageEntry>,      // pnpm
    pub bun_packages: Vec<PackageEntry>,       // bun
    pub pip_packages: Vec<PackageEntry>,       // pip
    pub cargo_packages: Vec<PackageEntry>,     // cargo
    pub brew_packages: Vec<PackageEntry>,      // brew
    pub custom_packages: HashMap<String, Vec<PackageEntry>>, // User-defined ✅
    // ... 15 more fields ...
}
```

### Problems

1. **Violates Open/Closed Principle**: Menambah backend baru requires **modifying struct**
2. **High Addition Cost**: 12+ files must change untuk add 1 backend
3. **Inconsistent Naming**: `packages` = AUR, yang lain use `*_packages`
4. **Iteration Complexity**: Must handle setiap vector secara manual
5. **Parser Duplication**: 10+ nearly identical parser implementations

---

## Decision

Refactor `RawConfig` untuk use **single HashMap untuk semua backends**:

```rust
pub struct RawConfig {
    pub imports: Vec<String>,
    
    // ✅ SINGLE HashMap untuk semua backends
    pub packages: HashMap<Backend, Vec<PackageEntry>>,
    
    // Custom backends tetap disini (user-defined)
    pub custom_packages: HashMap<String, Vec<PackageEntry>>,
    
    // ... other fields unchanged ...
}
```

### Backward Compatibility

Provide **accessor methods** untuk backward compatibility:

```rust
impl RawConfig {
    // Existing code can still use:
    // config.aur_packages() instead of config.packages.get(&Backend::Aur)
    
    pub fn aur_packages(&self) -> &Vec<PackageEntry> {
        self.packages.get(&Backend::Aur).unwrap_or(&EMPTY_VEC)
    }
    
    pub fn npm_packages(&self) -> &Vec<PackageEntry> {
        self.packages.get(&Backend::Npm).unwrap_or(&EMPTY_VEC)
    }
    
    // ... etc untuk all built-in backends
    
    // Unified iteration
    pub fn all_packages(&self) -> impl Iterator<Item = (&Backend, &Vec<PackageEntry>)> {
        self.packages.iter()
    }
}
```

### Backend Parser Unification

Replace 10+ individual parsers dengan **single generic parser**:

```rust
// Before: 10+ files like parsers/aur.rs, parsers/npm.rs, etc.

// After: Single generic parser
pub struct GenericBackendParser {
    backend: Backend,
    aliases: Vec<String>,
}

impl BackendParser for GenericBackendParser {
    fn backend(&self) -> &str { self.backend.as_str() }
    
    fn matches(&self, name: &str) -> bool {
        self.aliases.iter().any(|alias| alias == name)
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

// Registry initialization
impl BackendParserRegistry {
    pub fn new() -> Self {
        let mut parsers = HashMap::new();
        
        // All built-in backends use generic parser
        parsers.insert(Backend::Aur, Box::new(GenericBackendParser::new(
            Backend::Aur, 
            vec!["aur", "repo"]
        )));
        parsers.insert(Backend::Npm, Box::new(GenericBackendParser::new(
            Backend::Npm, 
            vec!["npm"]
        )));
        // ... etc
        
        Self { parsers }
    }
}
```

---

## Consequences

### Positive

1. **Open/Closed Principle Compliance**
   - Adding backend = add entry to HashMap, **no struct modification**

2. **Reduced Addition Cost**
   - Dari 12+ files ke **2-3 files**:
     1. Add Backend variant (core/types.rs)
     2. Register parser (kdl_modules/registry.rs)
     3. (Optional) Add custom implementation

3. **Unified Iteration**
   ```rust
   // Before: Manual iteration for each backend
   for pkg in &raw.packages { ... }
   for pkg in &raw.npm_packages { ... }
   for pkg in &raw.yarn_packages { ... }
   // ... 7 more loops
   
   // After: Single iteration
   for (backend, packages) in &raw.packages {
       for pkg in packages {
           process(pkg, backend);
       }
   }
   ```

4. **Reduced Parser Code**
   - -10 parser files (-600 lines)
   - Single generic implementation

5. **Type Safety**
   - Backend enum ensures type-safe access
   - No string-based lookups untuk built-in backends

### Negative

1. **Breaking API Change**
   - RawConfig field access changes from `raw.packages` ke `raw.packages.get(&Backend::Aur)`
   - **Mitigation:** Accessor methods untuk backward compatibility

2. **Migration Effort**
   - All code accessing `raw.npm_packages`, `raw.yarn_packages`, etc. must be updated
   - **Mitigation:** Mechanical changes, bisa automated dengan search/replace

3. **HashMap Overhead**
   - Minimal: HashMap dengan ~10 entries sangat efficient
   - Access time: O(1) vs direct field access O(1)

### Neutral

1. **Serialization**
   - serde support untuk HashMap<Backend, Vec<...>> works out of the box
   - State file format tetap compatible

---

## Implementation Plan

### Phase 2A.1: Update RawConfig (1-2 days)
- [ ] Refactor struct definition
- [ ] Add accessor methods untuk backward compatibility
- [ ] Implement Default untuk initialize all backends

### Phase 2A.2: Create Generic Parser (1-2 days)
- [ ] Implement GenericBackendParser
- [ ] Update BackendParserRegistry untuk use HashMap
- [ ] Register all built-in backends dengan generic parser

### Phase 2A.3: Update Existing Parsers (1 day)
- [ ] Remove old individual parsers (or keep untuk complex cases only)
- [ ] Update parser.rs untuk use new registry

### Phase 2A.4: Update All Access Sites (2-3 days)
- [ ] Update loader.rs untuk use unified iteration
- [ ] Update commands/check.rs
- [ ] Update commands/install.rs
- [ ] Update core/matcher.rs
- [ ] Update config/kdl_modules/registry.rs (inline prefix parsing)
- [ ] Update any other files accessing `raw.*_packages`

### Phase 2A.5: Testing (1-2 days)
- [ ] Run all 159 tests
- [ ] Add tests untuk new accessor methods
- [ ] Test backward compatibility accessors
- [ ] Performance benchmark (iteration speed)

---

## Migration Guide for Code Changes

### Access Pattern Changes

```rust
// Before
for pkg in &raw.packages { ... }              // AUR
for pkg in &raw.npm_packages { ... }          // npm
for pkg in &raw.yarn_packages { ... }         // yarn

// After - Option 1: Direct HashMap access
for pkg in raw.packages.get(&Backend::Aur).unwrap_or(&EMPTY_VEC) { ... }
for pkg in raw.packages.get(&Backend::Npm).unwrap_or(&EMPTY_VEC) { ... }

// After - Option 2: Accessor methods (recommended)
for pkg in raw.aur_packages() { ... }
for pkg in raw.npm_packages() { ... }

// After - Option 3: Unified iteration (for cross-backend processing)
for (backend, packages) in raw.all_packages() {
    for pkg in packages {
        process(pkg, backend);
    }
}
```

### Insertion Pattern Changes

```rust
// Before
raw.packages.push(entry);                    // AUR
raw.npm_packages.push(entry);                // npm

// After
raw.packages.entry(Backend::Aur).or_default().push(entry);
raw.packages.entry(Backend::Npm).or_default().push(entry);
```

---

## Alternatives Considered

### Alternative 1: Keep Current Structure
**Decision:** Rejected  
**Reason:** Violates OCP, high maintenance cost, inconsistent naming.

### Alternative 2: Use Vec<BackendConfig> Instead of HashMap
**Decision:** Rejected  
**Reason:** HashMap provides O(1) lookup by Backend enum, lebih type-safe dan efficient.

### Alternative 3: Macro-Based Field Generation
**Decision:** Rejected  
**Reason:** Macros would reduce boilerplate tapi add complexity dan reduce readability.

---

## References

- [Phase 2 Refactoring Plan](/docs/plans/PHASE2-REFACTOR-PLAN.md)
- [RawConfig Current Implementation](/src/config/kdl_modules/types.rs)
- [Backend Parser Registry](/src/config/kdl_modules/registry.rs)
- [Backend Enum](/src/core/types.rs)

---

## Success Metrics

1. **Files to Modify untuk New Backend**: < 3 files (was 12+)
2. **RawConfig Field Count**: ~12 fields (was 25+)
3. **Parser Implementations**: 1 generic + 0-3 custom (was 10+)
4. **Test Pass Rate**: 100% (159 tests)
5. **Performance**: No regression pada config loading

---

## Notes

- **Breaking Changes:** Internal API only. User-facing KDL config format unchanged.
- **Rollback Plan:** Revert RawConfig struct dan restore old parsers.
- **Testing Strategy:** Comprehensive test coverage sebelum migration, validate all access sites.
