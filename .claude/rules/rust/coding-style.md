# Rust Coding Style — mmap-guard

> Crate-specific delta only. **Authoritative source for this crate:
> `AGENTS.md` + `GOTCHAS.md`.**

## Toolchain

- Edition **2024**, stable toolchain, MSRV **1.85** (`rust-version` in `Cargo.toml`).
- Format with `cargo fmt`; lint via `just lint` (Rust, GitHub Actions, docs, and justfile lints).
- `just ci-check` is full local CI parity.

## Error handling (differs from generic ECC advice)

- This crate returns **`std::io::Error`** (`io::Result<_>`) — it does **not** use
  `thiserror` or `anyhow`. Do not add either. Deps are `memmap2` + `fs4` only.
- `unwrap_used` and `panic` are clippy **`deny`** (hard build failure), not advice.
  Use `?`. `expect_used` is `warn` — prefer `?` there too.
- Full pedantic/nursery/cargo clippy groups are on and promoted to deny via
  `-D warnings`. New code may trip lints you didn't opt into. See
  **GOTCHAS.md § Clippy Lints** before silencing anything (e.g. `indexing_slicing`,
  `unseparated_literal_suffix`, `uninlined_format_args`, `multiple_crate_versions`).

## Ownership & API

- Borrow (`&[u8]`, `&str`) by default; the crate exposes data via `Deref<Target=[u8]>`
  and `AsRef<[u8]>`.
- All public functions carry `#[must_use]` — maintain this for any new public API.
