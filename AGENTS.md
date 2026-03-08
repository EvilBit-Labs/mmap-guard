# AGENTS.md

This file provides guidance to AI coding assistants when working with code in this repository.

## Project Overview

`mmap-guard` is a Rust library that wraps `memmap2::Mmap::map()` behind a safe API, so downstream crates can use `#![forbid(unsafe_code)]` while still benefiting from zero-copy file access. See KICKOFF.md for the full design spec and API sketch.

Key design constraints:

- This crate **contains** the single `unsafe` block (the `memmap2` call) — it is NOT `#![forbid(unsafe_code)]`
- Must enforce `#![deny(clippy::undocumented_unsafe_blocks)]`
- Read-only mappings only; no mutable/writable mappings
- Rust edition 2024, stable toolchain

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

## Architecture

The crate is a library (`src/lib.rs`). Per KICKOFF.md, the target API is:

- **`FileData` enum** — `Mapped(Mmap)` | `Loaded(Vec<u8>)`, implements `Deref<Target = [u8]>` and `AsRef<[u8]>`
- **`map_file(path)`** — opens file, stats it, verifies non-empty, memory-maps it
- **`load(path)`** — memory-maps files, heap-loads stdin/pipes
- **`load_stdin()`** — reads stdin into a heap buffer

The only runtime dependency should be `memmap2`. Dev-dependency: `tempfile`.
