# Common Mistakes

## 1) Forgetting backend prefix on install

Symptom:

- package lands in unexpected backend block, or
- install command fails validation.

Fix:

```bash
declarch install aur:bat
# or
declarch install bat --backend aur
```

## 2) Running `sync` directly on first setup

Symptom:

- unexpected changes, or confusion about what will happen.

Fix: always preview first.

```bash
declarch sync preview
declarch sync
```

## 3) Backend configured but binary not installed

Symptom:

- errors like `Package manager error: <binary> not found`.

Fix:

```bash
command -v <binary>
```

Install missing binary or temporarily remove/disable that backend.

## 4) Editing KDL manually without validation

Symptom:

- parse error at sync/lint time.

Fix:

```bash
declarch lint --mode validate
```

## 5) Confusing config path vs state path

Symptom:

- editing one file while declarch reads another path.

Fix:

```bash
declarch info --doctor
```

Use the reported paths as source of truth.

## 6) Trying too many advanced flags at once

Symptom:

- hard to debug whether issue is from profile/host/modules/hooks.

Fix:

- start with plain `declarch sync preview`
- add one flag at a time (`--profile`, then `--host`, then modules)
