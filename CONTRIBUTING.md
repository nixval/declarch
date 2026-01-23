# Contributing to Declarch

Refactoring and contributing are welcome! Please follow these guidelines to keep the codebase clean and secure.

## Development Setup

1. **Install Rust**: Ensure you have the latest stable Rust toolchain.
2. **Clone**: `git clone https://github.com/nixval/declarch.git`
3. **Test**: `cargo test`

## Code Style

- Run `cargo fmt` before committing.
- Run `cargo clippy` and ensure there are no warnings.
- Use meaningful variable names and comments for complex logic.

## Security

- **Input Validation**: Always validate external input (package names, URLs, etc.).
- **Shell Execution**: Avoid `sh -c` where possible. Use `std::process::Command` with argument arrays.
- **Hook Safety**: Hooks are dangerous by design; ensure we warn users and execute them safely.

## Pull Requests

1. Create a descriptive branch name (e.g., `feat/add-dnf-support`).
2. Add tests for new features.
3. Update documentation if behavior changes.
4. Ensure CI passes.

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
