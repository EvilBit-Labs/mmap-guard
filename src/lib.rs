//! Safe, guarded memory-mapped file I/O for Rust.
//!
//! A thin wrapper around [`memmap2`] that encapsulates the single `unsafe` call
//! behind a safe API, so downstream crates can use `#![forbid(unsafe_code)]`
//! while still benefiting from zero-copy file access.
//!
//! # Overview
//!
//! This crate provides [`FileData`], an enum that transparently holds either a
//! memory-mapped file or a heap-allocated buffer. Both variants deref to `&[u8]`,
//! so consumers never need to know which backing store is in use.
//!
//! # Examples
//!
//! ```no_run
//! use mmap_guard::map_file;
//!
//! let data = map_file("large-file.bin").unwrap();
//! assert!(!data.is_empty());
//! // data derefs to &[u8] — use it like any byte slice
//! ```
//!
//! For CLI tools that accept both file paths and stdin, pass the user-supplied
//! path straight to [`load()`] — it routes `"-"` to stdin internally:
//!
//! ```no_run
//! use mmap_guard::load;
//!
//! // In a real CLI you'd get `path` from clap / std::env::args.
//! let path = std::env::args().nth(1).unwrap_or_else(|| "-".to_string());
//!
//! // No manual `if path == "-"` branch needed — `load` handles the dispatch.
//! let data = load(&path)?;
//! assert!(!data.is_empty());
//! # Ok::<(), std::io::Error>(())
//! ```
//!
//! If you need a custom byte cap for stdin, call [`load_stdin`] directly
//! (e.g. `load_stdin(Some(10 * 1024 * 1024))` for a 10 MiB limit).
//!
//! # Safety
//!
//! This crate contains exactly **one** `unsafe` block: the call to
//! [`memmap2::Mmap::map()`]. See the [`map_file`] documentation for the full
//! safety argument.
//!
//! # Known Limitations
//!
//! Advisory locks acquired by this crate are **cooperative**: they are only
//! effective when all processes accessing the same file honour the `fs4`
//! locking protocol. A process that truncates a file without acquiring the
//! advisory lock first may cause the OS to deliver `SIGBUS` (Unix) or an
//! access violation (Windows) when a mapped region is read. This is inherent
//! to memory-mapped I/O and cannot be fully eliminated.
//!
//! See [`map_file`]'s *Known Limitations* section for the full detail.

#![deny(clippy::undocumented_unsafe_blocks)]

mod file_data;
mod load;
mod map;

pub use file_data::FileData;
pub use load::{load, load_stdin};
pub use map::map_file;

// Re-export internals for fuzz targets when the `__fuzz` feature is active.
// This is NOT part of the public API — the leading underscores signal that.
#[cfg(feature = "__fuzz")]
#[doc(hidden)]
pub use load::read_bounded;
