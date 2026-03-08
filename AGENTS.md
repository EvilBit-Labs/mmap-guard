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

- `.github/CODEOWNERS` assigns `@unclesp1d3r` as reviewer for `*.rs` files only (not `Cargo.toml`/`Cargo.lock`, to avoid blocking dependabot).
- Mergify merge queue is enabled for bot PRs (dependabot, dosubot, release-plz). Human PRs are not auto-queued.
- `FileData` has a compile-time `Send + Sync` assertion (const block in `file_data.rs`) -- do not remove it; it guards against regressions if variant types change.
- All public functions (`map_file`, `load`, `load_stdin`) carry `#[must_use]` -- maintain this for any new public API.

The crate is a thin library with four source files:

- `src/lib.rs` — crate-level docs, re-exports public API
- `src/file_data.rs` — `FileData` enum (`Mapped(Mmap, File)` | `Loaded(Vec<u8>)`), `Deref<Target=[u8]>`, `AsRef<[u8]>`
- `src/map.rs` — `map_file()` with pre-flight stat check; contains the single `unsafe` block
- `src/load.rs` — `load()` routes `"-"` to `load_stdin(Some(1 GiB))`; other paths to `map_file()`. `load_stdin(max_bytes)` reads stdin into a heap buffer with optional byte cap

Runtime dependencies: `memmap2`, `fs4` (advisory file locking). Dev-dependencies: `tempfile`, `proptest`.

## Fuzzing & Property Tests

Coverage-guided fuzzing via `cargo-fuzz` (nightly) and property tests via `proptest` (stable).

### Fuzz targets (`fuzz/`)

The `fuzz/` directory is a separate Cargo workspace (not published). It depends on `mmap-guard` with the `__fuzz` feature to access internal functions.

```bash
# Install cargo-fuzz (one-time)
cargo install cargo-fuzz --locked

# Run a fuzz target (nightly required)
cargo +nightly fuzz run fuzz_read_bounded -- -max_total_time=60
cargo +nightly fuzz run fuzz_map_file -- -max_total_time=60

# List available targets
cargo +nightly fuzz list
```

Targets:

- `fuzz_read_bounded` — structured input (`Arbitrary`) exercising the bounded-read logic with fuzzer-controlled data and cap
- `fuzz_map_file` — writes fuzzer bytes to a temp file, maps it, asserts round-trip integrity

### Property tests

- `tests/prop_map_file.rs` — proptest integration test for `map_file` round-trip
- `src/load.rs` `mod tests::prop` — proptest for `read_bounded` (unit test, has access to private API)

### `__fuzz` feature flag

The `__fuzz` feature exposes `read_bounded` (normally private) as `#[doc(hidden)] pub`. It is not part of the public API — the leading underscores signal internal-only use. Only the fuzz crate enables it.

### CI workflows

- `.github/workflows/fuzz.yml` — weekly nightly fuzzing + merge queue gate, matrix over targets, uploads crash artifacts on failure
- `.github/workflows/compat.yml` — weekly Rust version compatibility matrix (stable, stable minus 2, stable minus 5, MSRV 1.85) + merge queue gate, runs build + tests with default features
- Both fuzz and compat workflows use the two-step CI pattern: they trigger on `pull_request` but skip on regular PRs via `if: startsWith(github.head_ref, 'mergify/merge-queue/')`. Mergify's `merge_conditions` use `check-success-or-neutral` so skipped jobs pass on regular PRs but block merge if they fail in the queue.

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
