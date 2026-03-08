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
//! For CLI tools that accept both file paths and stdin:
//!
//! ```no_run
//! use mmap_guard::{load, load_stdin};
//! use std::path::Path;
//!
//! let path = Path::new("input.txt");
//! let data = if path == Path::new("-") {
//!     load_stdin()
//! } else {
//!     load(path)
//! };
//! ```
//!
//! # Safety
//!
//! This crate contains exactly **one** `unsafe` block: the call to
//! [`memmap2::Mmap::map()`]. See the [`map_file`] documentation for the full
//! safety argument.

#![deny(clippy::undocumented_unsafe_blocks)]

mod file_data;
mod load;
mod map;

pub use file_data::FileData;
pub use load::{load, load_stdin};
pub use map::map_file;
