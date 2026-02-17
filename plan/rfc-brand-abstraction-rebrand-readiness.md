# RFC: Brand Abstraction & Rebrand Readiness (Declarch -> Future Brand)

Status: Draft (analysis only, no implementation)
Target: v0.8.x hardening track
Owner: core/cli/release

## 1. Problem Statement

Saat ini identitas produk (`declarch`) tersebar langsung di banyak layer:
- binary/help text
- path config/state/lock
- env vars
- installer/release artifact naming
- workflow/release metadata
- error/help strings

Akibatnya, setiap rebrand akan menjadi perubahan besar lintas sistem. Jika nanti ganti brand lagi, biaya maintenance akan berulang tinggi.

## 2. Goals

1. Rebrand selanjutnya menjadi perubahan kecil dan terkontrol.
2. Menjaga backward compatibility user lama.
3. Memisahkan `internal stable id` dari `display brand`.
4. Mengurangi risiko error release/install saat rename.

## 3. Non-Goals

1. Tidak mengganti brand sekarang.
2. Tidak mengubah behavior package resolution/state semantics.
3. Tidak membahas redesign docs (docs dianggap templated/find-replace).

## 4. Design Principles

1. Single Source of Truth untuk branding.
2. Runtime compatibility-first: old name tetap jalan selama masa transisi.
3. Deterministic migration: idempotent, reversible, observable.
4. No silent destructive migration.

## 5. Proposed Architecture

## 5.1 Brand Model

Tambahkan model metadata branding terpusat, misalnya `src/brand.rs`:
- `product_display_name`: `Declarch`
- `binary_name`: `declarch`
- `binary_aliases`: `["decl"]` (dan nanti legacy alias)
- `stable_product_id`: `declarch` (internal key untuk path/state)
- `config_dir_name`: `declarch`
- `state_dir_name`: `declarch`
- `env_prefix`: `DECLARCH`
- `release_asset_prefix`: `declarch`
- `repo_slug`: `nixval/declarch`
- `registry_slug`: `nixval/declarch-packages`

Poin penting:
- `stable_product_id` jangan sering berubah.
- `display`/`binary` bisa berubah lebih fleksibel via compatibility layer.

## 5.2 Identity Access Layer

Semua komponen dilarang hardcode brand string; wajib melalui accessor:
- `brand::binary_name()`
- `brand::config_dir()`
- `brand::state_dir()`
- `brand::release_asset_name(target)`
- `brand::env_key("MCP_ALLOW_APPLY")`

## 5.3 Compatibility Matrix

Dukungan transisi dua arah:
- binary baru + binary lama (alias/symlink)
- env baru + env lama (new overrides old)
- path baru + path lama (resolver/migrator)

Contoh aturan:
- Jika path baru kosong dan path lama ada -> gunakan path lama (atau migrasi aman dengan backup).
- Jika keduanya ada -> prioritas path baru, tampilkan hint di verbose.

## 5.4 Migration Policy

Perubahan brand dibagi 3 release:
1. Phase A (introduce): tambahkan abstraction + compat, default masih brand lama.
2. Phase B (switch): default display/binary baru, compat lama tetap aktif.
3. Phase C (cleanup): optional removal compat lama (major release).

## 6. Detailed Scope (Code)

## 6.1 CLI Layer

- Clap command name/help/banner ambil dari brand model.
- Unknown-command hints pakai formatter brand.
- Global messages (`Run 'declarch ...'`) tidak hardcoded.

## 6.2 Paths & Storage

- `utils/paths` membaca `brand` metadata.
- Lock/state backup filenames include stable id abstraction.
- Add migration guard + dry-run info path mapping.

## 6.3 Env & Feature Flags

- Generic env resolver:
  - `LPM_*` (future) and `DECLARCH_*` (legacy)
  - precedence + conflict diagnostics in verbose.

## 6.4 Installers

- `install.sh`/`install.ps1` consume shared brand constants (generated/static map).
- Asset name and repo URL derived centrally.
- Optional install of legacy symlink.

## 6.5 CI/Release

- Release workflow asset naming from one variable.
- Tag/title generation independent from literal project name.
- Smoke install step validates URL pattern and extracted binary names.

## 6.6 Testing Infrastructure

- Replace hardcoded brand assertions dengan helper `assert_brand_refs(...)`.
- Add migration tests (old->new path/env).
- Add installer contract tests for asset naming.

## 7. Risk Assessment

1. Migration bug bisa membuat state tidak terbaca.
- Mitigasi: read-old fallback + backup-before-write + strict tests.

2. Alias conflict (`lpm` terlalu generik).
- Mitigasi: pilih nama unik, cek command conflict di Linux/macOS/Windows.

3. Partial rebrand meninggalkan hardcoded string tersembunyi.
- Mitigasi: lint rule/custom grep gate for forbidden literals.

4. Installer mismatch asset naming.
- Mitigasi: contract test antara release workflow dan installer URL template.

## 8. Rollout Plan

## Phase 0 - Preparation
- Inventory literal references di code/ci/scripts.
- Define canonical brand model + compatibility targets.

## Phase 1 - Abstraction Foundation
- Add `brand` module and replace path/env/name access points.
- No behavior change.

## Phase 2 - Compatibility Layer
- Add dual env/path/binary support and migration logs.
- Add comprehensive tests.

## Phase 3 - Rebrand Flip (optional future)
- Change display/binary defaults to new brand.
- Keep legacy alias active.

## Phase 4 - Stabilization
- CI contract checks, installer checks, upgrade notes.

## 9. Acceptance Criteria

1. Rebrand requires editing <= 3 core files + installer/release metadata.
2. Existing users can run old command/path/env without data loss.
3. Installer artifacts always match release naming contract.
4. `cargo test`, `cargo deny`, and smoke checks pass with both old/new identity paths.

## 10. Open Decisions

1. New public binary name candidates and collision check.
2. Whether `stable_product_id` should remain `declarch` permanently.
3. Deprecation timeline for old binary/env/path compatibility.

## 11. Suggested Candidate Names

- lazypkg (recommended)
- lazypm
- packlazy
- lpkg
- pakku

Note: hindari `lpm` jika ingin minim collision dengan package/tool lain.
