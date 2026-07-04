# Rust Hooks / Local Gates — mmap-guard

> Crate-specific delta only. **Authoritative source: `AGENTS.md` + `GOTCHAS.md`.**

## Entry point: `just`

All dev workflows go through `just` (see `justfile`):

- `just ci-check` — full local CI parity (fmt, clippy, test, audit, coverage)
- `just lint` / `just test` / `just coverage-check` / `just audit` / `just deny`

## Pre-commit

`.pre-commit-config.yaml` runs on every commit: `cargo fmt`, `cargo clippy -- -D warnings`,
`cargo check`, cargo-machete, cargo-audit, cargo-sort, and mdformat on markdown
(excludes `.claude/`).

- Never bypass with `--no-verify` or `--no-gpg-sign`.
- `mdformat` and `cargo-sort` reformat files on commit — when a commit is rejected,
  **re-stage** the reformatted files and make a new commit (do not amend). See
  **GOTCHAS.md § Pre-commit Hooks**.
