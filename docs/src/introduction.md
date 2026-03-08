# Introduction

**mmap-guard** is a safe, guarded memory-mapped file I/O library for Rust. It wraps [`memmap2::Mmap::map()`](https://docs.rs/memmap2) behind a safe API so downstream crates can use `#![forbid(unsafe_code)]` while still benefiting from zero-copy file access.

## Motivation

Projects that enforce `#![forbid(unsafe_code)]` cannot call `memmap2::Mmap::map()` directly because it is `unsafe`. The alternative — `std::fs::read()` — copies the entire file into heap memory, which is impractical for disk images and multi-gigabyte binaries.

**mmap-guard** bridges this gap: one crate owns the unsafe boundary, every consumer gets a safe API.

Beyond simply wrapping the `unsafe` call, the goal is **isolation**. By centralizing the unsafe boundary in a single, focused crate, we can concentrate testing, fuzzing, and hardening efforts on that one point. mmap-guard should provide all reasonable protections against common mmap threats — SIGBUS from file truncation, empty file panics, permission errors — so that consumers don't have to reason about them.

## What It Does

1. **Safe mmap construction** — wraps `memmap2::Mmap::map()` with pre-flight checks
2. **Platform quirk mitigation** — documents and (where possible) mitigates SIGBUS/access violations from file truncation during mapping
3. **Cooperative SIGBUS mitigation** — acquires a shared advisory lock via `fs4` before mapping, reducing the risk of concurrent truncation
4. **Unified read API** — returns `&[u8]` whether backed by mmap or a heap buffer (for stdin/non-seekable inputs)

## What It Does NOT Do

- Provide mutable/writable mappings
- Expose a general file-locking or concurrency API to callers
- Abstract over async I/O
- Implement its own mmap syscalls (delegates entirely to `memmap2`)

## License

Licensed under either of

- [Apache License, Version 2.0](https://github.com/EvilBit-Labs/mmap-guard/blob/main/LICENSE-APACHE)
- [MIT License](https://github.com/EvilBit-Labs/mmap-guard/blob/main/LICENSE-MIT)

at your option.
