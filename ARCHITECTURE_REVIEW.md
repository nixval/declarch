# Declarch Architecture Review & Improvement Plan

**Date:** 2026-02-09  
**Version:** v0.8.0  
**Lines of Code:** ~14,784 (84 Rust files)

---

## Executive Summary

Declarch has evolved into a functional declarative package manager with a solid generic backend architecture. However, several architectural issues, code quality concerns, and potential vulnerabilities have been identified that should be addressed to ensure long-term maintainability, security, and reliability.

**Severity Legend:**
- 🔴 **Critical** - Security vulnerability or data loss risk
- 🟠 **High** - Significant bug risk or maintainability issue
- 🟡 **Medium** - Code quality or architectural concern
- 🟢 **Low** - Minor improvement opportunity

---

## 1. 🔴 Critical Issues

### 1.1 Command Injection Vulnerabilities

**Location:** Multiple command execution points  
**Issue:** User input is interpolated into shell commands without proper sanitization.

```rust
// In backends/generic.rs
cmd.arg("sh").arg("-c").arg(cmd_str);  // cmd_str contains user input
```

**Impact:** Malicious package names like `$(rm -rf /)` could execute arbitrary commands.

**Fix:** 
- Use argument arrays instead of shell strings
- Implement strict input validation
- Escape shell metacharacters

### 1.2 Path Traversal in Config Loading

**Location:** `src/config/loader.rs`  
**Issue:** While there's a check for `..`, it's not comprehensive.

```rust
if import_str.contains("..") {  // Easily bypassed with ./../ or encoded paths
    return Err(...);
}
```

**Impact:** Could read arbitrary files via symlinks or encoded paths.

### 1.3 Unsafe `unwrap()` Usage in Production Code

**Count:** 118 instances across codebase  
**High-Risk Locations:**
- `src/commands/hooks.rs:114` - Regex compilation
- `src/commands/install.rs` - User input reading
- Multiple test files

**Impact:** Panics in production, potential DoS.

---

## 2. 🟠 High Priority Issues

### 2.1 Global Mutable State (Singleton Anti-Pattern)

**Location:** `src/backends/registry.rs`

```rust
static REGISTRY: OnceLock<Arc<Mutex<BackendRegistry>>> = OnceLock::new();
```

**Issues:**
- Makes testing difficult (tests can pollute each other)
- Hidden dependencies
- Hard to reason about state changes
- Potential deadlocks

**Fix:** Use dependency injection or explicit context passing.

### 2.2 Mixed Error Handling Patterns

**Issue:** Inconsistent error handling across modules:
- Some use ` DeclarchError::Other(String)`
- Some use specific error variants
- Some use `map_err` with custom messages
- Some use `?` with `From` implementations

**Impact:** Hard to match on specific errors, poor error messages for users.

### 2.3 Tight Coupling Between Config and State

**Location:** `src/state/io.rs` - State knows about resolver internals

```rust
let canonical_key = resolver::make_state_key(&canonical_id);
```

**Issue:** Circular dependencies between state and core modules.

### 2.4 Inconsistent Directory/Path Management

**Issue:** Paths are constructed in multiple places with hardcoded strings:
- `paths.rs` defines constants
- But also constructed inline in various places
- XDG vs. platform-specific directories not consistently handled

### 2.5 Lack of Input Validation

**Issue:** Many user inputs lack validation:
- Package names
- Backend names
- Module paths
- Import paths

**Impact:** Silent failures or undefined behavior with invalid input.

---

## 3. 🟡 Medium Priority Issues

### 3.1 Code Duplication

**Locations Found:**

1. **File Loading Logic** - `load_single_module` and `load_config_with_modules` in `sync/mod.rs` have nearly identical path resolution logic (lines 185-218 and 220-261)

2. **Backend Import Detection** - Multiple regex patterns for detecting imports across `init.rs`

3. **Prompt/Confirmation Logic** - Duplicated yes/no prompts in multiple commands

4. **Backend Loading** - Similar loading patterns in `registry.rs` and `user_parser.rs`

### 3.2 Mixed Output Responsibilities

**Issue:** Output handling is scattered:
- `src/ui/mod.rs` - UI utilities
- But also direct `println!`/`eprintln!` in 122 places
- Some commands handle their own output formatting

**Impact:** Inconsistent UI, hard to implement themes or silent mode.

### 3.3 Overly Complex Match Statements

**Location:** `src/cli/dispatcher.rs` (160+ lines of match arms)

**Issue:** Each command variant maps to another enum variant with nearly identical fields. This boilerplate could be generated or simplified.

### 3.4 Lack of Trait Abstractions

**Issue:** 
- Config loading could use a `ConfigLoader` trait
- State management could use a `StateStore` trait
- This would make testing easier

### 3.5 Inconsistent Module Structure

**Issues:**
- `commands/sync/` is a directory with submodules
- Other commands are single files
- `kdl_modules` vs `kdl` naming inconsistency

### 3.6 Unused/Dead Code

**Evidence:**
- `#[allow(dead_code)]` annotations in `init.rs`
- Many `pub` functions that may not be used externally
- Commented-out code blocks

---

## 4. 🟢 Low Priority / Technical Debt

### 4.1 Clippy Warnings

**Count:** 18 warnings  
**Types:**
- Collapsible if statements
- Empty lines after doc comments
- Unnecessary clones

### 4.2 Magic Numbers/Strings

**Examples:**
- `DEFAULT_COMMAND_TIMEOUT = Duration::from_secs(300)` - Should be configurable
- `"backends/"` - Hardcoded path prefix
- Indentation strings `"    "` scattered in code

### 4.3 Missing Documentation

**Files with < 50% documentation coverage:**
- `src/backends/user_parser.rs` (953 lines)
- `src/commands/edit.rs`
- Several submodules

### 4.4 Test Organization

**Issues:**
- Unit tests in same files (good) but also integration tests mixed in
- Test helpers scattered
- Mock implementations not standardized

---

## 5. Architecture Improvements

### 5.1 Recommended: Clean Architecture / Hexagonal

```
┌─────────────────────────────────────────┐
│              Application Layer          │
│         (Commands, CLI handlers)        │
├─────────────────────────────────────────┤
│              Domain Layer               │
│    (Core types, PackageId, Backend)     │
├─────────────────────────────────────────┤
│            Infrastructure Layer         │
│  (Config loading, State I/O, Backends)  │
└─────────────────────────────────────────┘
```

**Benefits:**
- Clear dependency direction
- Easy to test (mock interfaces)
- Swappable implementations

### 5.2 Recommended: Command Pattern for CLI

Instead of large match statements:

```rust
trait Command {
    fn execute(&self, ctx: &Context) -> Result<()>;
    fn validate(&self) -> Result<()>;
}
```

### 5.3 Recommended: Dependency Injection

Replace global registry with explicit context:

```rust
struct AppContext {
    backend_registry: BackendRegistry,
    state_store: Box<dyn StateStore>,
    config_loader: Box<dyn ConfigLoader>,
}
```

---

## 6. Security Hardening

### 6.1 Input Validation Checklist

| Input Type | Validation Needed |
|------------|-------------------|
| Package names | Alphanumeric + limited special chars |
| Backend names | Strict whitelist |
| Import paths | Canonicalize, sandbox to config dir |
| Module names | No path separators |
| Shell commands | Escape or use execve |

### 6.2 Sandboxing Recommendations

- Use `std::process::Command` with args array instead of `sh -c`
- Implement seccomp-bpf for backend subprocesses
- Consider landlock for file system sandboxing

---

## 7. Performance Optimizations

### 7.1 Current Bottlenecks

1. **Backend Loading** - Reloaded on every command instead of cached
2. **Config Parsing** - Full tree loaded when only partial needed
3. **List Operations** - Synchronous shell commands block

### 7.2 Recommendations

- Implement lazy loading for backends
- Cache parsed configs with file-watcher invalidation
- Parallel backend operations where possible

---

## 8. Refactoring Plan

### Phase 1: Security (Immediate - 1 week)
1. Replace all shell command construction with safe arrays
2. Fix path traversal validation
3. Replace critical `unwrap()` calls with proper error handling

### Phase 2: Architecture (2-3 weeks)
1. Extract traits for ConfigLoader, StateStore
2. Remove global registry singleton
3. Implement dependency injection context

### Phase 3: Code Quality (Ongoing)
1. Eliminate code duplication
2. Fix clippy warnings
3. Improve documentation

### Phase 4: Testing (1-2 weeks)
1. Create mock implementations for traits
2. Add integration tests
3. Achieve > 80% coverage

---

## 9. Specific File Recommendations

| File | Issue | Priority |
|------|-------|----------|
| `backends/generic.rs` | Command injection risk | 🔴 |
| `config/loader.rs` | Path traversal | 🔴 |
| `backends/registry.rs` | Global singleton | 🟠 |
| `commands/sync/mod.rs` | Code duplication | 🟡 |
| `cli/dispatcher.rs` | Complexity | 🟡 |
| `state/io.rs` | Mixed concerns | 🟡 |
| `ui/mod.rs` | Scattered output | 🟢 |

---

## 10. Success Metrics

After improvements:
- Zero `unwrap()` in production code
- 100% safe command construction
- < 100 lines per function (currently several > 200)
- Zero clippy warnings
- > 80% test coverage
- All user inputs validated

---

## Appendix: Quick Fixes

### A.1 Replace shell command execution

```rust
// Before (DANGEROUS)
let cmd_str = format!("{} install {}", binary, packages.join(" "));
Command::new("sh").arg("-c").arg(cmd_str)

// After (SAFE)
let mut cmd = Command::new(binary);
cmd.arg("install");
for pkg in packages {
    cmd.arg(pkg);  // No shell interpretation
}
```

### A.2 Remove global state

```rust
// Before
static REGISTRY: OnceLock<...> = OnceLock::new();

// After
struct App {
    registry: BackendRegistry,
}
```

### A.3 Unify error handling

```rust
// Use thiserror properly
#[derive(Error, Debug)]
pub enum DeclarchError {
    #[error("Invalid package name: {0}")]
    InvalidPackageName(String),
    
    #[error("Backend not found: {0}")]
    BackendNotFound(String),
    
    // ... specific errors, not just Other(String)
}
```

---

**Reviewers:** Claude Code Analysis  
**Next Review Date:** After Phase 2 completion
