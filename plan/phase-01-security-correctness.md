# Phase 01 - Security & Correctness Hardening

## Objective
Menutup potensi bug/security yang paling berisiko tanpa mengubah UX besar.

## TODO
- [x] Perbaiki escaping shell agar sesuai implementasi single-quote escaping yang benar.
- [x] Tambah test edge-case untuk escaping/sanitization (quote, whitespace, symbol boundary).
- [x] Review validation path:
  - [x] package name validation
  - [x] search query validation
  - [x] hook command validation
- [ ] Hardening remote fetch URL:
  - [x] review skema allowed
  - [x] perketat private/local network block
  - [x] dokumentasikan trade-off compatibility
- [x] Audit input parsing CLI untuk fail-fast pada invalid user input (contoh numeric parse).
- [x] Verifikasi semua path error memberi pesan actionable.

## Progress Log

- 2026-02-15
  - fixed shell escaping for single quote path + regression test.
  - added edge-case tests for shell escaping (whitespace, symbol, passthrough safe chars).
  - changed search `--limit` parsing to fail-fast on invalid values.
  - fixed private `172.16.0.0/12` host range check (`16..=31`) + regression assert.
  - tightened remote URL validation with explicit malformed-host error path.
  - changed remote URL policy: `https` default; `http` requires `DECLARCH_ALLOW_INSECURE_HTTP=1`.
  - improved fetch failure diagnostics by returning compact attempted URL + reason summary.

## Compatibility Notes (Remote URL Policy)

- Before:
  - remote fetch accepted `http` and `https`.
- Now:
  - `https` is allowed by default.
  - `http` is blocked unless user explicitly sets env var:
    - `DECLARCH_ALLOW_INSECURE_HTTP=1`
- Trade-off:
  - security improves (safer default, less MITM exposure for remote init/fetch).
  - compatibility impact exists for legacy/self-hosted plain-HTTP endpoints.
  - mitigation is explicit, local opt-in via env var above.

## Actionable Error Path Verification Scope

- Verified and improved:
  - invalid search numeric limit (`--limit`) now fails with explicit usage hint.
  - malformed/blocked URL scheme now reports reason and opt-in path for insecure HTTP.
  - fetch failures now include attempted URL summary to shorten troubleshooting loop.

## Exit Criteria
- Temuan security/correctness prioritas ditutup.
- Test tambahan untuk area kritikal tersedia.
