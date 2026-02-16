# Phase 04 - Beginner Experience

## Objective
Mempermudah first-time user dan kontributor baru memahami workflow.

## TODO
- [x] Perbaiki pesan error yang ambigu menjadi langkah konkret.
- [x] Sederhanakan command help untuk flow umum.
- [x] Tambah dokumentasi quickstart linear:
  - [x] install
  - [x] init
  - [x] install packages
  - [x] sync
  - [x] troubleshoot basic failures
- [x] Tambah contoh konfigurasi minimal + advanced secara bertahap.
- [x] Tambah section "common mistakes".

## Progress Log

- 2026-02-16
  - improved beginner-oriented CLI guidance:
    - root command now shows quick-start steps via `--help` footer.
    - `sync --help` now includes most-common flow examples.
    - no-command path now prints actionable first-run command sequence.
    - improved `init --list` invalid target error with exact valid examples.
    - improved `--output-version v1` unsupported-command error with concrete supported commands.
  - added onboarding docs:
    - `docs-book/src/getting-started/first-run-linear.md`
    - `docs-book/src/getting-started/config-progression.md`
    - `docs-book/src/getting-started/common-mistakes.md`
  - updated navigation/entry points:
    - `docs-book/src/SUMMARY.md`
    - `docs-book/src/getting-started/quick-start.md`
    - `README.md` docs links

## Exit Criteria
- Jalur onboarding baru lebih pendek dan jelas.
- FAQ kesalahan umum terdokumentasi.
