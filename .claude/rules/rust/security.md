# Rust Security — mmap-guard

> Crate-specific delta only. **Authoritative source: `AGENTS.md` + `GOTCHAS.md`
> + `SECURITY.md`.** The generic ECC Rust-security topics (SQL injection, secret
> management, HTTP error responses, `tracing`) do **not** apply — this crate has no
> SQL, no secrets, and no network surface.

## The unsafe boundary (this crate's entire threat model)

- There must be **exactly one** `unsafe` block in the whole crate — the `memmap2`
  call in `src/map.rs`. No new `unsafe` without an issue discussion first.
- `undocumented_unsafe_blocks = "deny"` — every `unsafe` block needs a `// SAFETY:`
  comment proving the invariants hold.
- The crate is **not** `#![forbid(unsafe_code)]` — it *is* the boundary that lets
  downstream consumers forbid unsafe. See **GOTCHAS.md § Unsafe Code**.

## mmap hardening (what the safe API guarantees)

- Empty files are rejected by a deliberate pre-flight check (cannot be mapped).
- A shared advisory lock is taken via `fs4::FileExt::try_lock_shared()` before
  mapping; contention returns `WouldBlock`. Match all three arms (`Ok`, `WouldBlock`,
  `Error(io)`).
- SIGBUS from concurrent truncation is a **documented, out-of-scope limitation** — do
  not claim to fully prevent it. See **GOTCHAS.md § Platform / mmap** and `SECURITY.md`.

## Dependency & supply-chain checks

- Run `just audit` (`cargo audit`) and `just deny` (`cargo deny check`) locally
  before pushing. `just audit` is part of `just ci-check`; `cargo deny check` runs
  in the scheduled `security.yml` workflow — neither is a hard merge/release gate.
- The known `getrandom` duplicate is intentionally tolerated — see
  **GOTCHAS.md § Clippy Lints** before touching `deny.toml` or the
  `multiple_crate_versions` level.
