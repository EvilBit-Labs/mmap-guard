# Safety Contract

This crate exists to **isolate** the single `unsafe` operation behind a hardened boundary. By centralizing it here, we can focus testing, fuzzing, and defensive checks on this one point — so every downstream consumer benefits from those protections without reasoning about mmap safety themselves.

## The Unsafe Block

The crate contains exactly **one** `unsafe` block in `src/map.rs`:

```rust,ignore
// SAFETY: The file is opened read-only — no mutable aliasing is possible.
// A shared advisory lock is acquired before mapping to reduce (though not
// eliminate) the SIGBUS risk from concurrent truncation. Both the `Mmap`
// and the lock-owning `File` are moved into `FileData::Mapped`, ensuring
// the lock and mapping live and die together. Callers receive `&[u8]` with
// a lifetime tied to `FileData`, preventing use-after-unmap.
let mmap = unsafe { Mmap::map(&file)? };
```

## Safety Invariants

The safety of `memmap2::Mmap::map()` relies on these conditions, all of which mmap-guard upholds:

| Invariant                   | How it's upheld                                                                                                                                |
| --------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------- |
| File opened read-only       | `File::open()` opens in read-only mode                                                                                                         |
| File descriptor stays alive | `File` is kept alive by the caller through `FileData`                                                                                          |
| No use-after-unmap          | `&[u8]` lifetime is tied to `FileData` via `Deref`                                                                                             |
| No mutable aliasing         | Only read-only mappings are created                                                                                                            |
| Advisory lock held          | `fs4::FileExt::try_lock_shared` is called before mapping; the lock-owning `File` lives inside `FileData::Mapped` for the full mapping lifetime |

## Known Limitation: SIGBUS / Access Violation

If the underlying file is **truncated or modified by another process** while mapped, the operating system may deliver:

- **Unix:** `SIGBUS` signal
- **Windows:** Access violation (structured exception)

This is inherent to memory-mapped I/O. The advisory lock acquired by `map_file` mitigates but does not eliminate this risk, since non-cooperating processes may ignore the lock. The OS kernel does not provide a way to atomically verify file integrity while reading from a mapping.

### Mitigation Strategies

For applications that need robustness against concurrent file modification:

1. **Advisory locking** — mmap-guard acquires a cooperative shared lock via `fs4::FileExt::try_lock_shared` before creating the mapping. This is advisory only — it relies on other processes cooperating. If the lock cannot be acquired (another process holds an exclusive lock), `map_file` returns `io::ErrorKind::WouldBlock`.
2. **Signal handling** — install a `SIGBUS` handler that can recover gracefully (complex and platform-specific).
3. **Copy-on-read** — for small files, prefer `std::fs::read()` via the `FileData::Loaded` path.

## Why Not `#![forbid(unsafe_code)]`?

This crate is the **unsafe boundary** — it exists specifically to contain the one `unsafe` call that downstream `#![forbid(unsafe_code)]` crates cannot make themselves. Instead, the crate enforces:

- `#![deny(clippy::undocumented_unsafe_blocks)]` — every unsafe block must have a `// SAFETY:` comment
- Comprehensive clippy lints including `pedantic`, `nursery`, and security-focused rules
- The `unsafe` block count is maintained at exactly **one**
