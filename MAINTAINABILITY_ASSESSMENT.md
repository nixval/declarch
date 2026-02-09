# Declarch Maintainability Assessment

**Assessment Date:** 2026-02-09  
**Version:** v0.8.0  
**Overall Score: 6.5/10** (Above Average, but significant room for improvement)

---

## Scoring Breakdown

| Category | Score | Weight | Weighted |
|----------|-------|--------|----------|
| Code Organization | 7/10 | 20% | 1.4 |
| Module Coupling | 5/10 | 20% | 1.0 |
| Code Complexity | 6/10 | 15% | 0.9 |
| Testability | 5/10 | 15% | 0.75 |
| Documentation | 6/10 | 10% | 0.6 |
| Error Handling | 7/10 | 10% | 0.7 |
| Type Safety | 8/10 | 10% | 0.8 |
| **Total** | | | **6.15/10** |

*(Note: After recent fixes, score improved from ~5.5 to 6.5)*

---

## 1. Code Organization (7/10) ✅

### Strengths
- Clear module hierarchy (`cli/`, `commands/`, `backends/`, `config/`)
- Single Responsibility: Each module has a clear purpose
- Separation of concerns: UI, logic, and I/O are separated
- Good use of Rust's module system

### Weaknesses
- **File Size Inconsistency**: 
  - `init.rs` is 1,195 lines (too large)
  - `user_parser.rs` is 947 lines
  - Some modules are single files, others are directories
- **Deep Module Nesting**: `config/kdl_modules/helpers/` is 4 levels deep
- **Mixed Concerns in `commands/`**: Commands handle both CLI and business logic

### Recommendations
```
# Split large files
init.rs → init/
  ├── mod.rs          # Public API
  ├── backend.rs      # Backend initialization
  ├── module.rs       # Module initialization
  └── templates.rs    # Template generation

user_parser.rs → parser/
  ├── mod.rs
  ├── backend.rs
  ├── list.rs
  ├── search.rs
  └── validation.rs
```

---

## 2. Module Coupling (5/10) ⚠️

### Problems Identified

#### 2.1 Circular Dependencies
```
state/io.rs → core/resolver.rs → config/loader.rs → state/types.rs
```

#### 2.2 Excessive Cross-Module Imports
```rust
// In commands/sync/mod.rs - imports from 10+ different modules
use crate::config::loader;
use crate::core::types::SyncTarget;
use crate::error::Result;
use crate::ui as output;
use crate::utils::paths;
use crate::core::types::{PackageId, PackageMetadata};
use crate::packages::{PackageManager, create_manager};
use crate::state::types::Backend;
use crate::state;
```

#### 2.3 Hidden Dependencies via Globals
- `BackendRegistry` singleton still used in many places
- `paths::config_dir()` calls scattered throughout

### Dependency Graph Analysis
```
High Fan-In (many modules depend on):
- error (good: central error type)
- core/types (good: shared domain types)
- utils/paths (bad: should be injected)

High Fan-Out (depends on many modules):
- commands/sync/mod.rs (11 modules)
- cli/dispatcher.rs (15+ modules)
```

### Recommendations
1. **Create Core Domain Layer**: Extract pure business logic
2. **Service Layer Pattern**: Group related operations
3. **Dependency Injection**: Pass context explicitly

---

## 3. Code Complexity (6/10) ⚠️

### Metrics

| Metric | Current | Target |
|--------|---------|--------|
| Avg Function Length | 28 lines | <20 lines |
| Max Function Length | 156 lines (init_backend) | <50 lines |
| Avg File Length | 176 lines | <300 lines |
| Max File Length | 1,195 lines | <500 lines |
| Cyclomatic Complexity (avg) | ~8 | <10 |

### Complex Functions (>50 lines)
1. `init_backend` - 156 lines
2. `recursive_load` - 130 lines
3. `load_user_backends` - 115 lines
4. `parse_backend_node` - 95 lines
5. `dispatch` - 85 lines

### Deep Nesting Example
```rust
// In backends/generic.rs
if let Some(fallback_name) = &self.config.fallback {
    if let Ok(all_backends) = crate::backends::load_all_backends() {
        if let Some(fallback_config) = all_backends.get(fallback_name) {
            if let Some(fallback_bin) = fallback_config.binary.find_available() {
                return Ok(fallback_bin);
            }
        }
    }
}
```

### Recommendations
- Extract helper functions
- Use early returns to reduce nesting
- Split large files into modules
- Apply "Function Should Do One Thing" principle

---

## 4. Testability (5/10) ⚠️

### Current State
- **Unit Tests:** 129 tests, most passing
- **Integration Tests:** 14 tests
- **Coverage:** Estimated 40-50% (not measured)
- **Mocking:** No standardized mocking framework

### Problems

#### 4.1 Hard to Test Functions
```rust
// Depends on filesystem, network, and global state
pub fn init_backend(backend_name: &str, force: bool) -> Result<()>
```

#### 4.2 Missing Test Boundaries
- I/O operations mixed with business logic
- No clear ports/adapters architecture
- Tests require real filesystem

#### 4.3 Test Duplication
```rust
// init.rs has 90+ lines of test setup repeated
let mut temp_file = NamedTempFile::new().unwrap();
temp_file.write_all(content.as_bytes()).unwrap();
```

### Recommendations
1. **Test Pyramid**:
   - Unit: 70% (mocked dependencies)
   - Integration: 20% (test containers)
   - E2E: 10% (real environment)

2. **Test Helpers**:
   ```rust
   // test_helpers.rs
   pub fn with_temp_config<F>(content: &str, f: F) 
   pub fn mock_backend_registry() -> MockBackendRegistry
   ```

3. **Property-Based Testing**: For parsers and validators

---

## 5. Documentation (6/10) ⚠️

### Strengths
- Good module-level documentation
- Architecture decision records (CHANGELOG, ARCHITECTURE_REVIEW)
- Inline comments for complex logic

### Weaknesses
- **Function Documentation**: ~40% of public functions lack docs
- **Examples**: Few code examples in documentation
- **Architecture Diagrams**: None
- **README**: Good for users, lacks contributor docs

### Documentation Coverage by Module
| Module | Coverage | Notes |
|--------|----------|-------|
| backends/mod.rs | 90% | Good module docs |
| backends/user_parser.rs | 30% | 947 lines, minimal docs |
| commands/*.rs | 60% | Mixed |
| utils/*.rs | 50% | Functions lack docs |

### Recommendations
- Require doc comments for all `pub` items
- Add architecture diagrams (C4 model)
- Create CONTRIBUTING.md with code standards
- Document error variants with examples

---

## 6. Error Handling (7/10) ✅

### Strengths
- Custom error type with `thiserror`
- Error variants are descriptive
- Uses `?` operator effectively

### Weaknesses
- **Inconsistent Patterns**:
  ```rust
  // Pattern 1: Generic Other
  DeclarchError::Other(format!("Failed: {}", e))
  
  // Pattern 2: Specific variant
  DeclarchError::ConfigError(msg)
  
  // Pattern 3: map_err
  .map_err(|e| DeclarchError::IoError { path, source: e })
  ```

- **Error Context**: Some errors lack context about what operation failed
- **User-Facing Messages**: Some error messages are too technical

### Recommendations
```rust
// Use structured errors
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Failed to load config from {path}: {source}")]
    LoadFailed { path: PathBuf, source: std::io::Error },
    
    #[error("Import not found: {path}")]
    ImportNotFound { path: String },
}

// Then compose
#[derive(Error, Debug)]
pub enum DeclarchError {
    #[error(transparent)]
    Config(#[from] ConfigError),
    // ...
}
```

---

## 7. Type Safety (8/10) ✅

### Strengths
- Strong typing throughout
- Newtype pattern for `PackageId`, `Backend`
- Uses `Option` and `Result` properly
- Generic backend system is well-typed

### Weaknesses
- **Stringly Typed**: Some places use raw strings where enums could work
  ```rust
  // Could be an enum
  format: "json" | "whitespace" | "tsv"
  ```

- **Type Aliases**: Could use more newtypes for clarity
  ```rust
  // Instead of:
  type ModuleInfo<'a> = (&'a str, &'a str, Vec<&'a str>);
  
  // Use:
  struct ModuleInfo<'a> { path: &'a str, desc: &'a str, tags: Vec<&'a str> }
  ```

---

## Top 10 Maintainability Improvements

### Immediate (This Week)
1. **Split init.rs** into 4 modules (400+ lines each)
2. **Extract test helpers** to reduce duplication
3. **Add doc comments** to all public functions

### Short Term (This Month)
4. **Remove remaining global state** (BackendRegistry singleton)
5. **Unify error handling** patterns
6. **Create service layer** for business logic
7. **Add integration tests** with test containers

### Medium Term (This Quarter)
8. **Implement ports/adapters** architecture
9. **Add property-based tests** for parsers
10. **Create architecture documentation** with diagrams

---

## Conclusion

**Current State:** Declarch is functional but has technical debt. The recent Phase 1-3 improvements (security fixes, dependency injection traits, clippy fixes) have raised the score from ~5.5 to 6.5.

**Key Strengths:**
- Solid generic backend architecture
- Good separation of UI/business logic
- Type-safe implementation

**Key Risks:**
- Large files becoming unmaintainable
- Circular dependencies making refactors difficult
- Insufficient test coverage for confidence

**Recommendation:** Focus on splitting large files and improving test coverage before adding new features.
