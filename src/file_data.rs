//! The [`FileData`] enum — a unified read-only view over memory-mapped and
//! heap-allocated file data.

use std::fs::File;
use std::ops::Deref;

use memmap2::Mmap;

/// File data backed by either a memory map or a heap buffer.
///
/// Both variants dereference to `&[u8]`, so consumers can treat the data
/// uniformly regardless of the backing store.
///
/// # Errors
///
/// This type does not produce errors directly. Errors arise from the
/// functions that construct it — see [`map_file`](crate::map_file),
/// [`load()`](crate::load()), and [`load_stdin`](crate::load_stdin).
///
/// # Compatibility
///
/// This enum is `#[non_exhaustive]`, so match arms must include a wildcard.
/// The `Mapped` variant carries **two** fields — the memory map and the
/// file handle (for advisory locking). Always pattern-match with `..`
/// (e.g., `FileData::Mapped(..)`) rather than a fixed number of fields,
/// so your code remains forward-compatible if additional fields are added.
///
/// # Examples
///
/// ```no_run
/// use mmap_guard::map_file;
///
/// let data = map_file("example.bin").unwrap();
/// let bytes: &[u8] = &data;
/// println!("first byte: {:#04x}", bytes[0]);
/// ```
#[derive(Debug)]
#[non_exhaustive]
pub enum FileData {
    /// Data backed by a read-only memory map (zero-copy).
    ///
    /// The [`File`] handle is retained to hold the advisory lock for the
    /// lifetime of the map. Always match with `Mapped(..)` for forward
    /// compatibility.
    Mapped(Mmap, File),
    /// Data loaded into a heap-allocated buffer.
    Loaded(Vec<u8>),
}

impl Deref for FileData {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        match self {
            Self::Mapped(mmap, _file) => mmap,
            Self::Loaded(vec) => vec,
        }
    }
}

impl AsRef<[u8]> for FileData {
    fn as_ref(&self) -> &[u8] {
        self
    }
}

// Compile-time assertions: FileData must be Send + Sync so it can be shared
// across threads (Mmap and File are both Send + Sync).
// LCOV_EXCL_START — compile-time only, never called at runtime
const _: () = {
    const fn assert_send_sync<T: Send + Sync>() {}
    #[allow(dead_code)]
    const fn check() {
        assert_send_sync::<FileData>();
    }
};
// LCOV_EXCL_STOP

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loaded_variant_derefs_to_bytes() {
        let data = FileData::Loaded(vec![0xDE, 0xAD, 0xBE, 0xEF]);
        let bytes: &[u8] = &data;
        assert_eq!(bytes, &[0xDE, 0xAD, 0xBE, 0xEF]);
    }

    #[test]
    fn loaded_variant_as_ref() {
        let data = FileData::Loaded(vec![1, 2, 3]);
        let bytes: &[u8] = data.as_ref();
        assert_eq!(bytes, &[1, 2, 3]);
    }

    #[test]
    fn empty_loaded_variant() {
        let data = FileData::Loaded(vec![]);
        assert!(data.is_empty());
        assert_eq!(data.len(), 0);
    }
}
