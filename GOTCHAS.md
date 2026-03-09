# GOTCHAS.md

Hard-won lessons, edge cases, and "watch out for" patterns. Organized by domain.

Referenced from [AGENTS.md](AGENTS.md) and [CONTRIBUTING.md](CONTRIBUTING.md) -- read the relevant section before working in that area.

## Unsafe Code

- There must be exactly **one** `unsafe` block in the entire crate (in `src/map.rs`). Do not add new ones without an issue discussion first.
- `#![deny(clippy::undocumented_unsafe_blocks)]` is enforced -- every `unsafe` block must have a `// SAFETY:` comment explaining why the invariants are upheld.
- The crate is NOT `#![forbid(unsafe_code)]` -- it IS the unsafe boundary. Downstream consumers use `#![forbid(unsafe_code)]` and depend on this crate to encapsulate the unsafe call.

## Clippy Lints

- `missing_const_for_fn` (nursery, promoted to deny via `-D warnings`) -- functions inside `const` blocks (e.g., compile-time `Send + Sync` assertions) must be marked `const fn`.
- `option_if_let_else` (nursery, promoted to deny via `-D warnings`) -- prefer `Option::map_or` / `map_or_else` over `match` on `Option` for simple transformations.
- `indexing_slicing` = **warn** (promoted to deny via `-D warnings` in CI) -- direct slice indexing (e.g., `chunk[..n]`) is rejected. Use `#[allow(clippy::indexing_slicing)]` with a justification comment when bounds are provably safe; `.get()` can cause borrow-checker issues with mutable slices.
- `unseparated_literal_suffix` = **warn** (promoted to deny via `-D warnings` in CI) -- literal suffixes must use underscore separation (`0_u8`, not `0u8`).
- `multiple_crate_versions` = **warn** -- `fs4` and `tempfile` pull different `windows-sys` versions. The justfile `lint-rust` / `lint-rust-min` recipes pass `-A clippy::multiple_crate_versions` after `-D warnings` to prevent over-promotion. Do not change the Cargo.toml level to `deny` or `allow`.
- The same `windows-sys` duplication causes `cargo deny check` to fail on the `bans` policy. `deny.toml` has a `skip` entry for `windows-sys` -- keep it until `fs4` and `rustix` converge on the same version.
- `unwrap_used` = **deny**, `panic` = **deny** -- these fail the build in library code. Use `?` or proper error handling.
- `expect_used` = **warn** -- prefer `?` over `.expect()` in library code.
- Test modules need `#[allow(clippy::unwrap_used, clippy::expect_used)]` on the `mod tests` block.
- Full pedantic/nursery/cargo groups are enabled -- new code may trigger unexpected warnings from lint groups you didn't explicitly enable.
- `uninlined_format_args` is denied (via pedantic) -- use `"{var}"` not `"{}", var` in format strings.
- `exit` = **deny** (via nursery) -- `std::process::exit()` in subprocess helper tests needs `#[allow(clippy::exit)]` on the test function.

## Rustdoc

- `load` is both a module name (`mod load`) and a re-exported function (`pub use load::load`). In doc comments from submodules, link with `crate::load()` (parens disambiguate to the function) -- bare `crate::load` errors as ambiguous.
- `cargo doc --document-private-items` is used in CI. Links to private modules (e.g., `[`map`]`) will error because they resolve only with `--document-private-items` but break without it. Link to public items instead (e.g., `[`map_file`]`).
- Redundant explicit link targets (e.g., `[`map_file`](crate::map_file)`) are denied. Let rustdoc resolve intra-doc links automatically.

## FileData Enum

- `FileData` is `#[non_exhaustive]` -- match arms must include a wildcard. Adding a variant is a non-breaking change.
- `FileData::Mapped(Mmap, File)` carries both the memory map and the file handle (for advisory locking). Use `..` in `matches!` patterns (e.g., `matches!(data, FileData::Mapped(..))`), not `_`.
- `FileData` must implement `Debug` (required by `unwrap_err()` in tests and generally expected for public types).
- Both `Deref<Target=[u8]>` and `AsRef<[u8]>` are implemented -- consumers should use `&*data` or `data.as_ref()` interchangeably.

## CI

- `Cargo.lock` is gitignored (library crate convention). Do not commit it -- release-plz will refuse to run if `Cargo.lock` is both committed and gitignored.
- Mergify `queue_rules` requires both `queue_conditions` and `merge_conditions`. `merge_method` belongs on `queue_rules`, not the `queue` action. Parallel checks are configured via `merge_queue.max_parallel_checks` at the top level, not inside `queue_rules`.
- The `Cargo.toml` `exclude` list controls what ships to crates.io. Keep it comprehensive -- CI config, tooling, and non-essential docs should be excluded. Run `cargo package --list --allow-dirty` to audit.
- cargo subcommands installed via mise (e.g., cargo-dist) must be invoked as standalone binaries (`dist plan`) not cargo subcommands (`cargo dist plan`) -- cargo can't find mise-managed subcommands.
- `cargo-dist` plan/build does nothing for a library crate (no binary targets). That's why `dist-plan` is excluded from `just ci-check`.
- Mergify merge protections evaluate from the **main branch** config, not the PR branch.
- The docs workflow builds rustdoc with `--document-private-items` -- see Rustdoc section above for link pitfalls.
- Always verify pinned action SHAs with `gh api repos/{owner}/{repo}/commits/{sha} --jq '.sha'` before using them. Do not fabricate SHAs.

## Local CI with `act`

- `act` defaults to `push` event -- schedule-only workflows need `workflow_dispatch` passed as the event argument.
- `act` Docker containers run as root -- Unix permission tests (e.g., `chmod 000` → expect `PermissionDenied`) false-positive because root bypasses file permission checks.
- Use `--container-architecture linux/amd64` on Apple Silicon to avoid image pull failures.

## Pre-commit Hooks

- `mdformat` reformats markdown on commit -- if your commit is rejected, re-stage the reformatted files and create a new commit. Do not amend.
- `cargo-sort` reorders and aligns keys in `Cargo.toml` -- same pattern: re-stage and recommit.
- The `.claude/` directory is excluded from mdformat.

## Platform / mmap

- `fs4::FileExt::try_lock_shared()` returns `Result<bool>`, NOT `Result<()>` -- `Ok(false)` means contention, not `Err`. Always `match` on the bool.
- `flock()` locks do not conflict within the same process on macOS -- lock contention tests must spawn a subprocess (e.g., `python3 -c "import fcntl; ..."`) to hold the exclusive lock.
- Empty files cannot be memory-mapped -- `map_file()` returns an error for zero-length files. This is a deliberate pre-flight check.
- SIGBUS from concurrent file truncation is a **known, documented limitation** -- it cannot be fully prevented without advisory file locking. It is explicitly out of scope for security reports (see SECURITY.md).
- `map_file()` acquires a shared advisory lock via `fs4::fs_std::FileExt::try_lock_shared()` before mapping. Lock contention returns `WouldBlock`. The lock is held by the `File` inside `FileData::Mapped` and released on drop.

## Fuzzing

- The `__fuzz` feature flag exposes `read_bounded` as `#[doc(hidden)] pub` for fuzz targets. Do not use this feature in production or library code.
- `read_bounded` is `pub fn` in `src/load.rs` but the module is private — it is only reachable outside the crate when re-exported via `#[cfg(feature = "__fuzz")]` in `lib.rs`.
- Fuzz targets live in `fuzz/` (separate workspace, edition 2021). They require nightly and `cargo-fuzz`.
- The `fuzz/Cargo.toml` uses `edition = "2021"` (not 2024) because `cargo-fuzz` / `libfuzzer-sys` requires nightly and edition 2021 avoids compatibility issues.
- Property tests (`proptest`) run on stable and are part of the normal test suite. The `read_bounded` proptest is a unit test inside `src/load.rs` (not in `tests/`) because it needs access to the private function.
- `rust-toolchain.toml` overrides `rustup default` -- CI workflows that need nightly must set `RUSTUP_TOOLCHAIN: nightly` as an env var on the run step, not just install the toolchain.
- `read_bounded` needs `#[allow(unreachable_pub)]` and `#[allow(clippy::missing_errors_doc)]` because it's `pub` (for re-export) in a private module -- clippy flags both even though the function is `#[doc(hidden)]`.
- `#[derive(Arbitrary)]` generates code referencing `arbitrary::` by path -- `use libfuzzer_sys::arbitrary::{self, Arbitrary}` requires the `self` import. Do not remove it; the derive macro will fail without it.

## load / load_stdin

- `load("-")` delegates to `load_stdin(Some(1_073_741_824))` (1 GiB default cap). Callers needing a custom limit should call `load_stdin(Some(n))` directly.
- `load_stdin(max_bytes)` takes `Option<usize>` -- `None` = unlimited, `Some(n)` = hard cap returning `InvalidData` on overflow.
- The bounded read uses a 1-byte probe at the cap boundary to distinguish exact-fit EOF from genuine overflow.
- Do not call `load("-")` in unit tests — it reads real process stdin, which may block or behave inconsistently across test runners. Use `read_bounded` with a `Cursor` to test the stdin data path, and `resolve_source` to test the routing logic.
- To integration-test `load("-")`, spawn the test binary as a subprocess with piped stdin and an env-var guard. The test harness writes its own output to stdout, so use a temp file (not stdout) for child-to-parent data transfer.
