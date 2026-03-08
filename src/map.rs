//! Memory-mapped file construction with pre-flight checks.
//!
//! # Safety argument
//!
//! The single `unsafe` block in this module calls [`memmap2::Mmap::map()`].
//! The safety invariants are upheld as follows:
//!
//! - The file is opened **read-only** — no mutable aliasing is possible through
//!   this crate.
//! - The [`std::fs::File`] handle is kept alive for the lifetime of the
//!   [`Mmap`] (both are moved into [`FileData::Mapped`]).
//! - Callers receive `&[u8]` with a lifetime tied to [`FileData`], preventing
//!   use-after-unmap.
//!
//! **Known limitation:** if the underlying file is truncated or modified by
//! another process while mapped, the OS may deliver SIGBUS (Unix) or an access
//! violation (Windows). This is inherent to memory-mapped I/O and cannot be
//! fully prevented without advisory locking.

use std::fs::File;
use std::io;
use std::path::Path;

use memmap2::Mmap;

use crate::FileData;

/// Memory-map a file for read-only access.
///
/// Opens the file at `path`, verifies it is non-empty via `fstat`, then creates
/// a read-only memory mapping. Returns [`FileData::Mapped`] on success.
///
/// # Errors
///
/// Returns [`io::Error`] if:
/// - The file cannot be opened (permissions, not found, etc.)
/// - The file metadata cannot be read
/// - The file is empty (length 0)
/// - The memory mapping fails
///
/// # Examples
///
/// ```no_run
/// use mmap_guard::map_file;
///
/// let data = map_file("/usr/share/dict/words")?;
/// println!("file size: {} bytes", data.len());
/// # Ok::<(), std::io::Error>(())
/// ```
pub fn map_file(path: impl AsRef<Path>) -> io::Result<FileData> {
    let path = path.as_ref();
    let file = File::open(path)?;
    let metadata = file.metadata()?;

    if metadata.len() == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("file is empty: {}", path.display()),
        ));
    }

    // SAFETY: The file is opened read-only above. The `File` handle remains
    // alive as long as the `Mmap` because both are owned by the caller via
    // the returned `FileData`. No mutable mapping is created.
    let mmap = unsafe { Mmap::map(&file)? };

    Ok(FileData::Mapped(mmap))
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use std::io::Write;

    use tempfile::NamedTempFile;

    use super::*;

    #[test]
    fn map_file_reads_content() {
        let mut tmp = NamedTempFile::new().expect("failed to create temp file");
        tmp.write_all(b"hello mmap").expect("failed to write");
        tmp.flush().expect("failed to flush");

        let data = map_file(tmp.path()).expect("map_file failed");
        assert_eq!(&*data, b"hello mmap");
    }

    #[test]
    fn map_file_rejects_empty() {
        let tmp = NamedTempFile::new().expect("failed to create temp file");

        let err = map_file(tmp.path()).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
        assert!(
            err.to_string().contains("empty"),
            "error should mention 'empty': {err}"
        );
    }

    #[test]
    fn map_file_rejects_nonexistent() {
        let err = map_file("/tmp/mmap_guard_does_not_exist_12345").unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::NotFound);
    }

    #[test]
    fn map_file_returns_mapped_variant() {
        let mut tmp = NamedTempFile::new().expect("failed to create temp file");
        tmp.write_all(b"data").expect("failed to write");
        tmp.flush().expect("failed to flush");

        let data = map_file(tmp.path()).expect("map_file failed");
        assert!(
            matches!(data, FileData::Mapped(_)),
            "expected Mapped variant"
        );
    }
}
