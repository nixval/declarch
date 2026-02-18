# First Run (Linear Guide)

Use this if you want one straight path from zero to first successful sync.

## 1) Install declarch

Choose one method from [Installation](./installation.md), then verify:

```bash
declarch --version
declarch --help
```

## 2) Initialize config

```bash
declarch init
```

If you are unsure where files are created on your OS:

```bash
declarch info --doctor
```

## 3) Add packages

Declarative-first path (recommended):

```kdl
// ~/.config/declarch/modules/base.kdl
pkg {
    aur { bat fd ripgrep }
    npm { typescript }
}
```

Then apply with preview:

```bash
declarch --dry-run sync
declarch sync
```

Direct install shortcut (still valid):

```bash
declarch install aur:bat aur:fd aur:ripgrep
declarch install npm:typescript
// check it at `decl edit others`
```

Alternative: use one backend for all packages in a command:

```bash
declarch install bat fzf ripgrep --backend aur
```

## 4) Preview changes

```bash
declarch --dry-run sync
```

Review what will be installed/adopted/removed.

## 5) Apply

```bash
declarch sync
```

## 6) First troubleshooting loop

If a command fails, run this sequence:

```bash
declarch lint --mode validate
declarch info --doctor
declarch --dry-run sync
```

Then check [Troubleshooting](../advanced/troubleshooting.md) for targeted fixes.
