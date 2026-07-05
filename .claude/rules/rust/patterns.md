# Rust Patterns — mmap-guard

> Crate-specific delta only. **Authoritative source: `AGENTS.md` + `GOTCHAS.md`.**

This is a thin, four-file library (`lib.rs`, `file_data.rs`, `map.rs`, `load.rs`).
Do **not** introduce repository/service-layer/API-envelope/builder scaffolding from
the generic ECC patterns — none of it applies to a mmap file-I/O boundary.

## The `FileData` enum (the one real domain type)

- `#[non_exhaustive]` → every `match` needs a wildcard arm; adding a variant is
  non-breaking.
- Variants: `Mapped(Mmap, File)` (map + file handle for advisory locking) and
  `Loaded(Vec<u8>)`. Use `..` in `matches!` (e.g. `matches!(d, FileData::Mapped(..))`),
  never `_`.
- Keep the compile-time `Send + Sync` const assertion in `file_data.rs` — it guards
  against regressions if variant types change. Do not remove it.
- Implements `Debug`, `Deref<Target=[u8]>`, and `AsRef<[u8]>`. See
  **GOTCHAS.md § FileData Enum**.

## Unsafe boundary

The single `unsafe` block lives in `src/map.rs`. Treat it as the crate's whole
reason to exist — see `security.md` and **GOTCHAS.md § Unsafe Code**.
