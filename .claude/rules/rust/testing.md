# Rust Testing — mmap-guard

> Crate-specific delta only. **Authoritative source: `AGENTS.md` + `GOTCHAS.md`.**

## Framework & commands

- Prefer **nextest**: `cargo nextest run` (single test: `cargo nextest run <name>`).
  Fallback: `cargo test`. `just test` wraps this.
- **proptest** (stable) drives property tests and runs in the normal suite. The
  `read_bounded` proptest is a unit test inside `src/load.rs` because it needs the
  private function; `tests/prop_map_file.rs` covers `map_file` round-trips.
- **cargo-fuzz** (nightly) drives the `fuzz/` targets (`fuzz_read_bounded`,
  `fuzz_map_file`). The `__fuzz` feature re-exports `read_bounded` for them. See
  **GOTCHAS.md § Fuzzing** — do not use `__fuzz` in library code.
- No async here → no `#[tokio::test]`. `rstest`/`mockall` are not used.

## Coverage

- `cargo llvm-cov`; the threshold is **85%** (`just coverage-check`), not 80%.

## Gotchas

- Test modules need `#[allow(clippy::unwrap_used, clippy::expect_used)]` on the
  `mod tests` block (both are otherwise denied/warned).
- Do **not** call `load("-")` in unit tests — it reads real process stdin. Test the
  data path with `read_bounded` + a `Cursor`, and routing with `resolve_source`.
  See **GOTCHAS.md § load / load_stdin**.
- Permission tests (`chmod 000` → `PermissionDenied`) false-pass as root — relevant
  under `act`. See **GOTCHAS.md § Local CI with act**.
