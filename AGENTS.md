# AGENTS.md

This file provides guidance to AI coding assistants when working with code in this repository.

@GOTCHAS.md

## Project Overview

`mmap-guard` is a Rust library that wraps `memmap2::Mmap::map()` behind a safe API, so downstream crates can use `#![forbid(unsafe_code)]` while still benefiting from zero-copy file access. See KICKOFF.md for the full design spec and API sketch.

The core motivation is **isolation**: by centralizing the unsafe boundary in a single, focused crate, we can concentrate testing, fuzzing, and hardening efforts on that one point. This library should provide all reasonable protections against common mmap threats (SIGBUS from truncation, empty files, permission errors) so consumers don't have to reason about them.

Key design constraints:

- This crate **contains** the single `unsafe` block (the `memmap2` call) — it is NOT `#![forbid(unsafe_code)]`
- Must enforce `#![deny(clippy::undocumented_unsafe_blocks)]`
- Read-only mappings only; no mutable/writable mappings
- The unsafe boundary should be exhaustively tested and hardened — prioritize safety coverage
- Rust edition 2024, stable toolchain
- See GOTCHAS.md for unsafe code rules and downstream expectations.

## Build & Development Commands

```bash
# Build
cargo build

# Test (prefer nextest)
cargo nextest run
cargo nextest run <test_name>    # single test
cargo test                        # fallback if nextest unavailable

# Lint
cargo fmt --check
cargo clippy -- -D warnings

# Coverage
cargo llvm-cov                    # requires cargo-llvm-cov via mise

# Security audits
cargo audit
cargo deny check

# Format
cargo fmt

# All tools managed via mise — run `mise install` to bootstrap
```

## Pre-commit Hooks

Pre-commit is configured (`.pre-commit-config.yaml`) and runs on commit:

- `cargo fmt`, `cargo clippy -- -D warnings`, `cargo check`
- cargo-machete (unused dependencies), cargo-audit, cargo-sort
- mdformat on markdown (excludes `.claude/`)
- See GOTCHAS.md for pre-commit re-staging pitfalls.

## Architecture

The crate is a thin library with four source files:

- `src/lib.rs` — crate-level docs, re-exports public API
- `src/file_data.rs` — `FileData` enum (`Mapped(Mmap)` | `Loaded(Vec<u8>)`), `Deref<Target=[u8]>`, `AsRef<[u8]>`
- `src/map.rs` — `map_file()` with pre-flight stat check; contains the single `unsafe` block
- `src/load.rs` — `load()` delegates to `map_file()`; `load_stdin()` reads into heap buffer

The only runtime dependency should be `memmap2`. Dev-dependency: `tempfile`.

## Lint Configuration

- Clippy denies `unwrap_used` and `panic`; warns on `expect_used` — test modules need `#[allow(clippy::unwrap_used, clippy::expect_used)]`
- `undocumented_unsafe_blocks = "deny"` — every `unsafe` block must have a `// SAFETY:` comment
- Full pedantic/nursery/cargo lint groups enabled (see `[workspace.lints.clippy]` in Cargo.toml)
- See GOTCHAS.md for clippy and rustdoc edge cases.

## Just Commands

All dev workflows use `just` (see `justfile`):

- `just ci-check` — full local CI parity (fmt, clippy, test, audit, coverage)
- `just test` / `just test-ci` — run nextest
- `just coverage` / `just coverage-check` — llvm-cov (85% threshold)
- `just lint` — fmt + clippy + actionlint + markdownlint
- `just audit` / `just deny` — security checks
- See GOTCHAS.md for CI and tooling edge cases.
