# Version Pinning (Planned)

Version pinning is not active yet.

Current direction:

- backend-capability based (not every package manager supports strict pinning)
- opt-in per package/backend
- clear handling for unsupported pin requests (`warn` or `error`)
- backward-compatible config/state evolution

When implemented, this page will include:

- supported backends/capability matrix
- syntax examples
- migration notes
