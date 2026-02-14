# KDL Basics

Declarch config uses KDL.
This page covers only beginner-level syntax.

## Minimal example

```kdl
pkg {
    pacman {
        firefox
        git
    }
}
```

## Rules you should remember

1. Use blocks with `{}`.
2. Package names are plain entries inside backend blocks.
3. Use quotes for string values in settings fields.

Example with quoted values:

```kdl
meta {
    title "My Setup"
    description "My daily packages"
}
```

## Common pattern

```kdl
imports {
    "modules/base.kdl"
    "modules/dev.kdl"
}

pkg {
    pacman { firefox }
    flatpak { org.mozilla.firefox }
    npm { typescript }
}
```

## Need full syntax details?

Use advanced reference:
- [Syntax Reference (Advanced)](./syntax.md)
