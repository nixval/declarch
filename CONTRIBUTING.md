# Contributing to Declarch

Thanks for your interest in contributing! 🎉

Please see the [full documentation](https://nixval.github.io/declarch/) for:

- **Development setup** - Getting started with the codebase
- **Code style guidelines** - Formatting and linting
- **Pull request process** - How to submit changes
- **Security considerations** - Input validation and hook safety

## Quick Start

1. **Install Rust**: Ensure you have the latest stable Rust toolchain
2. **Clone**: `git clone https://github.com/nixval/declarch.git`
3. **Test**: `cargo test --all-targets`
4. **Format**: `cargo fmt`
5. **Lint**: `cargo clippy --all-targets --all-features -- -D warnings`
6. **Dependency policy**: `cargo deny check`

If your environment exports `RUSTC_WRAPPER=sccache` and you hit permission errors, run with wrapper disabled:

- `RUSTC_WRAPPER= cargo test --all-targets`
- `RUSTC_WRAPPER= cargo clippy --all-targets --all-features -- -D warnings`
- `RUSTC_WRAPPER= cargo deny check`

## Changelog Discipline

If your change affects behavior, CLI UX, defaults, safety policy, or troubleshooting steps:

1. Add an entry under `## [Unreleased]` in `CHANGELOG.md`.
2. Keep entries user-facing (what changed, migration impact, and safe usage notes).
3. Keep technical implementation details in commit history or PR description, not changelog noise.

## Quick Links

- [Full Documentation](https://nixval.github.io/declarch/)
- [Configuration Reference](https://nixval.github.io/declarch/configuration/kdl-syntax.html)
- [Command Reference](https://nixval.github.io/declarch/commands/)

## License

By contributing, you agree that your contributions will be licensed under the [MIT License](LICENSE).
