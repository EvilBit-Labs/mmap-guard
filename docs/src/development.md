# Development Setup

## Prerequisites

All development tools are managed by [mise](https://mise.jdx.dev/). Install mise, then run:

```bash
just setup
```

This installs the Rust toolchain, cargo extensions (nextest, llvm-cov, audit, deny, etc.), and other tools defined in `mise.toml`.

## Common Commands

| Command                | Description                                       |
| ---------------------- | ------------------------------------------------- |
| `just build`           | Build the library                                 |
| `just test`            | Run tests with nextest                            |
| `just lint`            | Format check + clippy + actionlint + markdownlint |
| `just fmt`             | Format Rust code                                  |
| `just fix`             | Auto-fix clippy warnings                          |
| `just coverage`        | Generate LCOV coverage report                     |
| `just coverage-report` | Open HTML coverage report in browser              |
| `just audit`           | Run cargo-audit for vulnerabilities               |
| `just deny`            | Run cargo-deny for license/ban checks             |
| `just ci-check`        | Full local CI parity check                        |
| `just docs-build`      | Build mdBook + rustdoc                            |
| `just docs-serve`      | Serve docs locally with live reload               |

## Pre-commit Hooks

Pre-commit hooks run automatically on `git commit`:

- `cargo fmt` — code formatting
- `cargo clippy -- -D warnings` — lint checks
- `cargo check` — compilation check
- `cargo-machete` — unused dependency detection
- `cargo-audit` — security audit
- `cargo-sort` — Cargo.toml key ordering
- `mdformat` — markdown formatting

If the hooks modify files (e.g., formatting), re-stage and commit again.

## CI Pipeline

The GitHub Actions CI runs on every push to `main` and on pull requests:

1. **quality** — rustfmt + clippy
2. **test** — nextest + release build
3. **test-cross-platform** — Linux (x2), macOS, Windows
4. **coverage** — llvm-cov uploaded to Codecov
