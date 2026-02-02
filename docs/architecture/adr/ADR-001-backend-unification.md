# ADR-001: Backend Implementation Unification Strategy

**Status:** Accepted  
**Date:** 2026-02-02  
**Author:** @nixval (via kimioc)  
**Related:** ADR-002, ADR-003, PHASE2-REFACTOR-PLAN.md

---

## Context

Declarch saat ini memiliki **dua backend implementation patterns** yang berjalan paralel:

1. **Custom Implementations** (`src/packages/*.rs`): 9 backends dengan manual Rust code
2. **Generic Infrastructure** (`src/backends/`): Config-driven tapi hanya 1 backend (pip) yang menggunakannya

Ini menciptakan:
- Duplikasi logic dan maintenance burden
- Inconsistent behavior antar backends
- Confusion untuk contributors (kapan pakai custom vs generic?)
- Wasted complexity (generic infrastructure canggih tapi underused)

---

## Decision

Kita akan **unify backend implementations** ke generic infrastructure dengan custom implementations sebagai extension/override.

### Strategy: Generic-First dengan Custom Fallback

```
┌─────────────────────────────────────────┐
│         Backend Requested               │
└─────────────┬───────────────────────────┘
              │
              ▼
┌─────────────────────────────────────────┐
│   Is it a complex backend?              │
│   (AUR, Flatpak, Soar)                  │
└─────────────┬───────────────────────────┘
              │
      ┌───────┴───────┐
      │ YES           │ NO
      ▼               ▼
┌──────────────┐ ┌──────────────────────┐
│ Use Custom   │ │ Use GenericManager   │
│ Implementation│ │ with BackendConfig   │
└──────────────┘ └──────────────────────┘
```

### Simple Backends → Generic

Backends berikut akan **dimigrasi dari custom ke generic**:

| Backend | Current | Target | Reason |
|---------|---------|--------|--------|
| npm | Custom impl | Generic | Standard `list --json`, `install -g`, `uninstall -g` |
| yarn | Custom impl | Generic | Standard commands, can delegate list to npm |
| pnpm | Custom impl | Generic | Standard JSON output |
| bun | Custom impl | Generic | Standard JSON output |
| cargo | Custom impl | Generic | Simple whitespace output with regex |
| brew | Custom impl | Generic | Simple whitespace output |
| pip | Already Generic | Generic | No change needed |

### Complex Backends → Tetap Custom

| Backend | Strategy | Reason |
|---------|----------|--------|
| AUR | Custom implementation | Helper detection (paru/yay), special handling |
| Flatpak | Custom implementation | Remote management, complex tab-separated parsing |
| Soar | Custom implementation | Auto-installation, ANSI stripping, complex parsing |

### Custom → Can Extend Generic

Complex backends bisa extend `GenericManager` untuk reuse code:

```rust
pub struct AurManager {
    generic: GenericManager,  // Reuse for standard operations
    helper: String,           // AUR-specific: paru/yay
}

impl PackageManager for AurManager {
    fn list_installed(&self) -> Result<HashMap<String, PackageMetadata>> {
        // AUR-specific: use helper
        self.execute_via_helper(&self.generic.config.list_cmd)
    }
    
    // Other methods delegate to generic where possible
}
```

---

## Consequences

### Positive

1. **Reduced Code Duplication**
   - -6 implementation files (-600 lines)
   - Single source of truth untuk output parsing
   - Unified error handling

2. **Easier Backend Addition**
   - New backend = 1 config entry di `backends/registry.rs`
   - No new Rust code untuk simple backends
   - Consistent behavior out of the box

3. **Better Testability**
   - Mock BackendConfig untuk testing
   - No need to mock individual backend implementations

4. **Maintained Flexibility**
   - Custom implementations tetap tersedia untuk complex cases
   - Generic can be extended/overridden

### Negative

1. **Initial Migration Risk**
   - Need comprehensive testing untuk ensure no behavioral changes
   - Edge cases di custom implementations harus di-identify dan di-handle

2. **Configuration Complexity**
   - BackendConfig punya 30+ fields (tapi akan di-simplify di ADR-003)

3. **Learning Curve**
   - Contributors need to understand when to use generic vs custom

### Neutral

1. **Performance**
   - Minimal impact: GenericManager uses same command execution
   - Might even improve due to reduced code paths

---

## Implementation Plan

### Phase 2B.1: Enhance GenericManager (2-3 days)
- [ ] Add search support ke GenericManager
- [ ] Add `use_delegate_list` untuk backends yang share listing (yarn → npm)
- [ ] Add regex pattern support untuk line-by-line parsing (cargo)

### Phase 2B.2: Create BackendConfigs (2-3 days)
- [ ] Define BackendConfig untuk npm, yarn, pnpm, bun, cargo, brew
- [ ] Test each config dengan existing test suite
- [ ] Document config fields

### Phase 2B.3: Migrate Backends (2-3 days)
- [ ] Update BackendRegistry::register_defaults() untuk use GenericManager
- [ ] Remove old custom implementations
- [ ] Run full test suite

### Phase 2B.4: Validation (1-2 days)
- [ ] Integration testing untuk setiap backend
- [ ] Performance benchmark (before vs after)
- [ ] User acceptance testing

---

## Alternatives Considered

### Alternative 1: Keep Both Patterns
**Decision:** Rejected  
**Reason:** Maintains status quo dengan duplication dan confusion. Tidak solve masalah fundamental.

### Alternative 2: Remove Generic Infrastructure
**Decision:** Rejected  
**Reason:** Generic infrastructure bagus untuk simple backends dan user-defined backends. Removing it would increase boilerplate untuk new backends.

### Alternative 3: Code Generation (Macros)
**Decision:** Rejected (for now)  
**Reason:** Macros would reduce boilerplate tapi add complexity. Config-driven approach lebih readable dan maintainable.

---

## References

- [Original Analysis](/docs/plans/PHASE2-REFACTOR-PLAN.md)
- [Backend Registry](/src/packages/registry.rs)
- [Generic Manager](/src/backends/generic.rs)
- [Current Custom Implementations](/src/packages/)

---

## Notes

- **Breaking Changes:** None untuk end users. Internal API changes only.
- **Rollback Plan:** Revert registry.rs changes dan restore deleted files.
- **Monitoring:** Watch untuk issue reports related to backend behavior setelah release.
