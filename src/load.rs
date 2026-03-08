//! Convenience functions for loading data from files or stdin.
//!
//! [`load`] is the recommended entry point for CLI tools that accept both file
//! paths and piped input. It memory-maps regular files for zero-copy access and
//! falls back to heap allocation for non-seekable sources.

use std::io::{self, Read};
use std::path::Path;

use crate::file_data::FileData;
use crate::map::map_file;

/// Load data from a file path, using memory mapping when possible.
///
/// For regular files, this delegates to [`map_file`] for
/// zero-copy access. The path must point to a non-empty, readable file.
///
/// # Errors
///
/// Returns [`io::Error`] if the file cannot be opened, is empty, or cannot
/// be memory-mapped.
///
/// # Examples
///
/// ```no_run
/// use mmap_guard::load;
///
/// let data = load("input.bin")?;
/// println!("loaded {} bytes", data.len());
/// # Ok::<(), std::io::Error>(())
/// ```
pub fn load(path: impl AsRef<Path>) -> io::Result<FileData> {
    map_file(path)
}

/// Read all of stdin into a heap-allocated buffer.
///
/// Returns [`FileData::Loaded`] containing the complete contents of stdin.
/// This is useful for CLI tools that accept piped input via `-`.
///
/// # Errors
///
/// Returns [`io::Error`] if reading from stdin fails.
///
/// # Examples
///
/// ```no_run
/// use mmap_guard::load_stdin;
///
/// let data = load_stdin()?;
/// println!("read {} bytes from stdin", data.len());
/// # Ok::<(), std::io::Error>(())
/// ```
pub fn load_stdin() -> io::Result<FileData> {
    let mut buf = Vec::new();
    io::stdin().read_to_end(&mut buf)?;
    Ok(FileData::Loaded(buf))
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use std::io::Write;

    use tempfile::NamedTempFile;

    use super::*;

    #[test]
    fn load_maps_regular_file() {
        let mut tmp = NamedTempFile::new().expect("failed to create temp file");
        tmp.write_all(b"load test").expect("failed to write");
        tmp.flush().expect("failed to flush");

        let data = load(tmp.path()).expect("load failed");
        assert_eq!(&*data, b"load test");
        assert!(
            matches!(data, FileData::Mapped(..)),
            "expected Mapped variant for regular file"
        );
    }

    #[test]
    fn load_rejects_empty_file() {
        let tmp = NamedTempFile::new().expect("failed to create temp file");

        let err = load(tmp.path()).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidInput);
    }

    #[test]
    fn load_rejects_nonexistent_file() {
        let err = load("/tmp/mmap_guard_load_missing_12345").unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::NotFound);
    }
}
