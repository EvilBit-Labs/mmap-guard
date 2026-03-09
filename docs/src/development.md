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

### Running Tests

**Standard tests:**

```bash
just test
```

**Property-based tests:**

```bash
cargo test --test prop_map_file
```

Property tests use [proptest](https://github.com/proptest-rs/proptest) to verify round-trip integrity with randomized inputs.

**Fuzz tests:**

Fuzzing requires nightly Rust and `cargo-fuzz`. Install it first:

```bash
cargo install cargo-fuzz
```

Run a specific fuzz target (available targets: `fuzz_read_bounded`, `fuzz_map_file`):

```bash
cargo +nightly fuzz run fuzz_read_bounded
cargo +nightly fuzz run fuzz_map_file
```

Fuzz tests use the `__fuzz` feature flag to expose internal APIs for testing. This feature is for internal use only and should not be enabled in production code.

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

**Weekly scheduled workflows:**

- **fuzz** — runs fuzzing tests (`fuzz_read_bounded`, `fuzz_map_file`) with nightly Rust. Also runs on merge queue PRs.
- **compat** — tests Rust version compatibility across stable, stable-2, stable-5, and MSRV 1.85. Also runs on merge queue PRs.

These weekly workflows use `check-success-or-neutral` conditions for merge gating, allowing merges when the checks pass or are skipped.
