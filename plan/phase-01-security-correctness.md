# Phase 01 - Security & Correctness Hardening

## Objective
Menutup potensi bug/security yang paling berisiko tanpa mengubah UX besar.

## TODO
- [ ] Perbaiki escaping shell agar sesuai implementasi single-quote escaping yang benar.
- [ ] Tambah test edge-case untuk escaping/sanitization (quote, whitespace, symbol boundary).
- [ ] Review validation path:
  - [ ] package name validation
  - [ ] search query validation
  - [ ] hook command validation
- [ ] Hardening remote fetch URL:
  - [ ] review skema allowed
  - [ ] perketat private/local network block
  - [ ] dokumentasikan trade-off compatibility
- [ ] Audit input parsing CLI untuk fail-fast pada invalid user input (contoh numeric parse).
- [ ] Verifikasi semua path error memberi pesan actionable.

## Exit Criteria
- Temuan security/correctness prioritas ditutup.
- Test tambahan untuk area kritikal tersedia.
