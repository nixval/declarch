# KDL Basics

Declarch config is written in KDL.
This page only covers beginner syntax.

## Minimal example

```kdl
pkg {
    pacman {
        firefox
        git
    }
}
```

## Rules to remember

1. Blocks use `{}`.
2. Package names are plain entries inside backend blocks.
3. Quote string values in metadata/settings fields.

```kdl
meta {
    title "My Setup"
    description "My daily packages"
}
```

## Typical layout

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

Need full syntax details? Use [Syntax Reference (Advanced)](./syntax.md).
