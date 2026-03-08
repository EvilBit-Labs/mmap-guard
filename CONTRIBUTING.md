# Contributing to mmap-guard

Thank you for your interest in contributing to mmap-guard! This document provides guidelines and information for contributors.

## Code of Conduct

This project follows the [Contributor Covenant Code of Conduct](CODE_OF_CONDUCT.md). Please be respectful and constructive in all interactions.

## Getting Started

### Prerequisites

- **Rust 1.89+** (edition 2024, stable toolchain)
- **Git** for version control
- **[mise](https://mise.jdx.dev/)** for tool management (recommended)

### Quick Start

```bash
# Clone the repository
git clone https://github.com/EvilBit-Labs/mmap-guard.git
cd mmap-guard

# Install development tools
mise install

# Build the project
cargo build

# Run tests
cargo nextest run
```

## Development Setup

### Recommended Tools

All tools are managed via mise — run `mise install` to bootstrap:

- **cargo-nextest** — fast test runner
- **cargo-llvm-cov** — code coverage
- **cargo-audit** / **cargo-deny** — security auditing
- **cargo-about** — third-party license notices
- **just** — task runner
- **pre-commit** — git hooks
- **mdbook** — documentation

### Development Commands

```bash
# Full local CI check
just ci-check

# Test
just test                         # run nextest
cargo nextest run <test_name>     # single test

# Lint
just lint                         # fmt + clippy + actionlint + markdownlint
cargo fmt --check
cargo clippy -- -D warnings

# Coverage
just coverage                     # generate report
just coverage-check               # enforce 85% threshold

# Security
just audit                        # cargo audit
just deny                         # cargo deny check

# Documentation
cd docs && mdbook serve --open    # local preview
cargo doc --open                  # rustdoc
```

### Building Documentation

```bash
# Build and serve the mdbook documentation
cd docs
mdbook serve --open

# Generate rustdoc
cargo doc --open
```

## Architecture

mmap-guard is a thin library with four source files:

| Module             | Purpose                                                            |
| ------------------ | ------------------------------------------------------------------ |
| `src/lib.rs`       | Crate-level docs, re-exports public API                            |
| `src/map.rs`       | `map_file()` with pre-flight stat check; the single `unsafe` block |
| `src/load.rs`      | `load()` delegates to `map_file()`; `load_stdin()` reads to heap   |
| `src/file_data.rs` | `FileData` enum (`Mapped` / `Loaded`), `Deref`, `AsRef`            |

See [Architecture Documentation](docs/src/architecture.md) for details.

## Making Changes

### Branching Strategy

1. Create a feature branch from `main`:

   ```bash
   git checkout -b feat/your-feature-name
   ```

2. Use conventional commit prefixes:

   - `feat:` — New features
   - `fix:` — Bug fixes
   - `docs:` — Documentation changes
   - `refactor:` — Code refactoring
   - `test:` — Test additions/changes
   - `chore:` — Maintenance tasks
   - `perf:` — Performance improvements
   - `ci:` — CI/CD changes

### Code Quality Requirements

Before submitting changes, ensure:

1. **All tests pass**: `cargo nextest run`
2. **No clippy warnings**: `cargo clippy -- -D warnings`
3. **Code is formatted**: `cargo fmt`
4. **Documentation builds**: `cargo doc --no-deps`

### Safety Requirements

This crate **contains** the unsafe boundary — it is NOT `#![forbid(unsafe_code)]`. However:

- There must be exactly **one** `unsafe` block (the `memmap2::Mmap::map()` call)
- `#![deny(clippy::undocumented_unsafe_blocks)]` is enforced — every `unsafe` block must have a `// SAFETY:` comment
- Do not add new `unsafe` blocks without opening an issue for discussion first
- Read-only mappings only — no mutable or writable mappings

### Lint Configuration

The crate uses strict clippy linting:

- `unwrap_used` = **deny** — use `?` or proper error handling
- `panic` = **deny** — no panics in library code
- `expect_used` = **warn** — prefer `?` over `.expect()`
- Test modules need `#[allow(clippy::unwrap_used, clippy::expect_used)]`
- Full pedantic/nursery/cargo lint groups enabled

## Testing

### Running Tests

```bash
# Run all tests
cargo nextest run

# Run a specific test
cargo nextest run test_name

# Run with output
cargo nextest run -- --nocapture

# Check coverage
just coverage-check               # 85% threshold
```

### Writing Tests

- Place unit tests in the same file as the code being tested
- Use `#[cfg(test)]` modules with `#[allow(clippy::unwrap_used, clippy::expect_used)]`
- Include doc tests for public API examples
- Test both success and error cases

Example test structure:

```rust
#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_success() {
        let result = function_under_test(input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_feature_error() {
        let result = function_under_test("");
        assert!(result.is_err());
    }
}
```

## Documentation

### Types of Documentation

1. **Rustdoc** — API documentation in source code
2. **mdBook** — Developer guide in `docs/`
3. **README.md** — Project overview and quick start

### Rustdoc Guidelines

- Document all public items
- Include examples in doc comments with `# Examples` sections
- Add `# Errors` sections for fallible functions
- Add `# Panics` sections if applicable

## Submitting Changes

### Pull Request Process

1. **Update documentation** for any API changes
2. **Add tests** for new functionality
3. **Run the full check suite** locally: `just ci-check`
4. **Create a pull request** with a clear description
5. **Address review feedback** promptly

### Code Review Requirements

All pull requests require review before merging. Reviewers check for:

- **Correctness** — Does the code do what it claims? Are edge cases handled?
- **Safety** — No new unsafe blocks, proper bounds checking, no panics in library code
- **Tests** — New functionality has tests, existing tests still pass
- **Style** — Follows project conventions, passes `cargo fmt` and `cargo clippy -- -D warnings`
- **Documentation** — Public APIs have rustdoc with examples, AGENTS.md updated if architecture changes

CI checks run before merge, including quality checks, tests, coverage, and cross-platform tests (Ubuntu, macOS, Windows).

### Developer Certificate of Origin (DCO)

This project requires all contributors to sign off on their commits, certifying that they have the right to submit the code under the project's license. This is enforced by the [DCO GitHub App](https://github.com/apps/dco).

To sign off, add `-s` to your commit command:

```bash
git commit -s -m "feat: add new feature"
```

This adds a `Signed-off-by` line to your commit message:

```text
Signed-off-by: Your Name <your.email@example.com>
```

By signing off, you agree to the [Developer Certificate of Origin](https://developercertificate.org/).

### PR Description Template

```markdown
## Summary
Brief description of changes

## Changes
- Change 1
- Change 2

## Testing
How were these changes tested?

## Checklist
- [ ] Tests pass (`cargo nextest run`)
- [ ] No clippy warnings (`cargo clippy -- -D warnings`)
- [ ] Code formatted (`cargo fmt`)
- [ ] Documentation updated
- [ ] Commits signed off (`git commit -s`)
```

## Style Guidelines

### Rust Style

This project uses `rustfmt` with edition 2024. Run `cargo fmt` before committing.

### Error Handling

- Use `Result<T, E>` for fallible operations
- Use `std::io::Error` for I/O operations
- Provide context in error messages
- Never use `unwrap()` or `panic!()` in library code

## Project Governance

### Decision-Making

mmap-guard uses a **maintainer-driven** governance model. Decisions are made by the project maintainers through consensus on GitHub issues and pull requests.

### Roles

| Role            | Responsibilities                                                           | Current                                                                                        |
| --------------- | -------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------- |
| **Maintainer**  | Merge PRs, manage releases, set project direction, review security reports | [@unclesp1d3r](https://github.com/unclesp1d3r), [@KryptoKat08](https://github.com/KryptoKat08) |
| **Contributor** | Submit issues, PRs, and participate in discussions                         | Anyone following this guide                                                                    |

### How Decisions Are Made

- **Bug fixes and minor changes**: Any maintainer can review and merge
- **New features**: Discussed in a GitHub issue before implementation; maintainer approval required
- **Architecture changes**: Require agreement from both maintainers
- **Breaking API changes**: Discussed in a GitHub issue with community input; require agreement from both maintainers

### Becoming a Maintainer

As the project grows, active contributors who demonstrate sustained, high-quality contributions and alignment with project goals may be invited to become maintainers.

## Getting Help

- **Issues** — For bug reports and feature requests
- **Discussions** — For questions and ideas
- **Documentation** — Check [docs/](docs/) for detailed guides

Thank you for contributing to mmap-guard!
