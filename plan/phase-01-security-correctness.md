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
  - [ ] review skema allowed
  - [x] perketat private/local network block
  - [ ] dokumentasikan trade-off compatibility
- [x] Audit input parsing CLI untuk fail-fast pada invalid user input (contoh numeric parse).
- [ ] Verifikasi semua path error memberi pesan actionable.

## Progress Log

- 2026-02-15
  - fixed shell escaping for single quote path + regression test.
  - added edge-case tests for shell escaping (whitespace, symbol, passthrough safe chars).
  - changed search `--limit` parsing to fail-fast on invalid values.
  - fixed private `172.16.0.0/12` host range check (`16..=31`) + regression assert.
  - tightened remote URL validation with explicit malformed-host error path.

## Exit Criteria
- Temuan security/correctness prioritas ditutup.
- Test tambahan untuk area kritikal tersedia.
