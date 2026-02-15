# RFC-0003: Codebase Quality Stabilization Roadmap

- Status: Draft
- Authors: declarch team
- Created: 2026-02-16
- Target Version Window: v0.8.x -> v0.9.0

## 1. Summary

RFC ini mendefinisikan roadmap peningkatan kualitas codebase declarch secara bertahap dengan fokus pada:
- correctness dan security hardening,
- maintainability dan modularity,
- performance reliability,
- beginner-friendly developer experience,
- release guardrails untuk regression prevention.

Roadmap ini **tidak langsung mengubah behavior besar**. Prioritas awal adalah stabilisasi, pengurangan technical debt, dan memastikan perubahan mudah di-trace dengan commit kecil, terukur, dan mudah rollback.

## 2. Context

Observasi dari audit codebase saat ini:
- Codebase modular secara domain, tetapi beberapa file menjadi hotspot kompleksitas (>1k LOC).
- Test suite kuat dan cepat; baseline regresi cukup baik.
- Clippy strict (`-D warnings`) belum bersih.
- Ada area security/correctness yang harus diprioritaskan (escaping shell, URL hardening).
- CLI features cukup kaya namun ada potensi kebingungan untuk pengguna/pengembang baru.

## 3. Goals

1. Menaikkan kualitas keseluruhan ke level production-hardening tanpa rewrite total.
2. Menjadikan lint/test sebagai quality gate wajib.
3. Menurunkan kompleksitas file inti dan meningkatkan local reasoning.
4. Memastikan setiap perubahan memiliki jalur rollback sederhana.

## 4. Non-Goals

- Rewrite architecture besar-besaran.
- Menghapus fitur user-facing secara agresif tanpa migration path.
- Breaking changes besar tanpa fase deprecation.

## 5. Principles

1. Small-step commits, single concern per commit.
2. Refactor behavior-preserving dulu, behavior-changing belakangan.
3. Dead code hanya dihapus jika benar-benar tidak dipakai dan ada verifikasi lint/test.
4. Redundant logic boleh di-merge bila menurunkan kompleksitas tanpa menurunkan keterbacaan.
5. Semua phase punya exit criteria jelas.

## 6. Work Chapters

### Chapter A: Baseline & Safety Net
- Tetapkan baseline metrics (test time, clippy debt, hotspot LOC).
- Kunci quality gate minimal untuk mencegah debt bertambah.

### Chapter B: Security & Correctness
- Perbaikan escape/sanitization.
- Hardening URL/network policy.
- Error-handling dan input validation yang eksplisit.

### Chapter C: Maintainability & Modularity
- Pecah mega-file jadi submodule terarah.
- Kurangi branching complexity di orchestration/dispatcher.
- Rapikan type complexity dengan type alias / DTO.

### Chapter D: Performance & Reliability
- Profiling lightweight di path panas.
- Hindari retry/timeout pattern yang tersebar tanpa standar.
- Standarisasi observability minimal untuk operasi panjang.

### Chapter E: Beginner Experience
- CLI error message lebih actionable.
- Dokumentasi onboarding dan troubleshooting yang lebih linear.

### Chapter F: Release Guardrails
- CI matrix final (fmt, clippy, test).
- Tagging, rollback notes, dan changelog discipline.

## 7. Phase Plan

- Phase 00: Baseline & inventory
- Phase 01: Security & correctness hardening
- Phase 02: Maintainability & modularity refactor
- Phase 03: Performance & reliability improvements
- Phase 04: Beginner-friendly UX/docs improvements
- Phase 05: Release guardrails + stabilization

Detail TODO per phase ada di folder `plan/`.

## 8. Dead Code / Redundancy Policy

- Remove jika memenuhi semua syarat:
  1. tidak dipanggil lint/test/build graph,
  2. tidak bagian dari kontrak publik yang terdokumentasi,
  3. tidak dibutuhkan compatibility layer.
- Merge redundant logic jika:
  1. duplikasi nyata,
  2. behavior tetap,
  3. test coverage tetap/lengkap.

## 9. Risks

- Refactor modularisasi bisa memicu regressi behavior edge-case.
- Hardening security bisa memblokir use-case lama (perlu compatibility notes).
- Scope creep jika phase boundaries tidak disiplin.

Mitigasi: commit kecil, test gate tiap langkah, release notes incremental.

## 10. Exit Criteria

Roadmap dianggap selesai jika:
- clippy strict bersih,
- test suite hijau konsisten,
- hotspot kompleksitas utama terpecah,
- isu security/correctness prioritas terselesaikan,
- docs onboarding lebih jelas,
- release process punya guardrails yang repeatable.

## 11. Rollback Strategy

- Tiap chapter dikerjakan dalam batch commit kecil.
- Bila regressi, rollback by commit (non-interactive git) tanpa perlu revert seluruh phase.
- Jaga commit message berbasis scope agar tracing cepat.
