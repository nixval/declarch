# Commit Strategy

## Rules
- 1 commit = 1 concern.
- Refactor behavior-preserving dipisah dari behavior-changing.
- Jangan campur docs, refactor, dan fix kritikal dalam satu commit bila bisa dipisah.

## Commit Message Template
- `phase-XX(scope): short summary`

Contoh:
- `phase-01(security): fix shell escaping for package arguments`
- `phase-02(sync): extract planner helpers into submodule`
- `phase-03(search): measure and reduce backend fanout overhead`

## Safety Sequence per Commit
1. `cargo fmt --check`
2. `cargo test --all-targets`
3. Jalankan subset clippy sesuai scope, lalu full clippy pada milestone.

## Rollback
- Rollback by commit hash.
- Hindari amend untuk commit yang sudah jadi checkpoint fase.
