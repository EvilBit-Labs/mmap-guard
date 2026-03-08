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
//! **Known limitation:** a shared advisory lock is acquired before mapping
//! (via `fs4`), which mitigates SIGBUS from concurrent truncation when
//! cooperating processes also use advisory locks. However, advisory locks are
//! not mandatory — if another process truncates the file without checking the
//! lock, the OS may still deliver SIGBUS (Unix) or an access violation
//! (Windows). This is inherent to memory-mapped I/O.

use std::fs::File;
use std::io;
use std::path::Path;

use fs4::fs_std::FileExt;
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
/// - The file is empty (length 0) — [`io::ErrorKind::InvalidInput`]
/// - The file is exclusively locked by another process — [`io::ErrorKind::WouldBlock`]
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
            io::ErrorKind::InvalidInput,
            format!("file is empty: {}", path.display()),
        ));
    }

    match FileExt::try_lock_shared(&file) {
        Ok(true) => {} // Lock acquired successfully.
        Ok(false) => {
            return Err(io::Error::new(
                io::ErrorKind::WouldBlock,
                format!("file is locked by another process: {}", path.display()),
            ));
        }
        Err(e) => {
            return Err(io::Error::new(
                e.kind(),
                format!("failed to acquire shared lock on {}: {e}", path.display()),
            ));
        }
    }

    // SAFETY: The file is opened read-only — no mutable aliasing is possible.
    // A shared advisory lock is acquired before mapping to reduce (though not
    // eliminate) the SIGBUS risk from concurrent truncation. Both the `Mmap`
    // and the lock-owning `File` are moved into `FileData::Mapped`, ensuring
    // the lock and mapping live and die together. Callers receive `&[u8]` with
    // a lifetime tied to `FileData`, preventing use-after-unmap.
    let mmap = unsafe { Mmap::map(&file)? };

    Ok(FileData::Mapped(mmap, file))
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
        assert_eq!(err.kind(), io::ErrorKind::InvalidInput);
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
            matches!(data, FileData::Mapped(..)),
            "expected Mapped variant"
        );
    }

    #[cfg(unix)]
    #[test]
    fn map_file_returns_would_block_when_exclusively_locked() {
        use std::process::{Command, Stdio};

        let mut tmp = NamedTempFile::new().expect("failed to create temp file");
        tmp.write_all(b"locked data").expect("failed to write");
        tmp.flush().expect("failed to flush");

        let path = tmp.path().to_owned();

        // Spawn a child process that exclusively locks the file via
        // flock() and blocks on stdin. A separate process is required
        // because flock() locks are per-open-file-description, and
        // same-process locks on different FDs do not conflict on macOS.
        let mut child = Command::new("python3")
            .arg("-c")
            .arg(
                "import fcntl, os, sys; \
                 fd = os.open(sys.argv[1], os.O_RDONLY); \
                 fcntl.flock(fd, fcntl.LOCK_EX); \
                 sys.stdout.write('locked\\n'); sys.stdout.flush(); \
                 sys.stdin.readline()",
            )
            .arg(&path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("failed to spawn lock holder");

        // Wait for the child to signal that it holds the lock.
        let stdout = child.stdout.as_mut().expect("missing stdout");
        let mut buf = [0_u8; 7]; // "locked\n"
        io::Read::read_exact(stdout, &mut buf).expect("child did not signal");

        let err = map_file(&path).unwrap_err();
        assert_eq!(
            err.kind(),
            io::ErrorKind::WouldBlock,
            "expected WouldBlock, got: {err}"
        );
        assert!(
            err.to_string().contains(&path.display().to_string()),
            "error should mention the file path: {err}"
        );

        // Drop stdin to let the child exit, then reap it.
        drop(child.stdin.take());
        child.wait().expect("failed to reap child");
    }
}
