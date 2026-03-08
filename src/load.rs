//! Convenience functions for loading data from files or stdin.
//!
//! [`load`] is the recommended entry point for CLI tools that accept both file
//! paths and piped input. It memory-maps regular files for zero-copy access and
//! falls back to heap allocation for non-seekable sources.

use std::io::{self, Read};
use std::path::Path;

use crate::file_data::FileData;
use crate::map::map_file;

/// Default stdin byte limit used by [`load`] when path is `"-"` (1 GiB).
const DEFAULT_STDIN_MAX_BYTES: usize = 1_073_741_824;

/// Routing decision for [`load`].
#[derive(Debug, PartialEq, Eq)]
enum LoadSource<'a> {
    /// The path `"-"` was given — read from stdin.
    Stdin,
    /// A regular file path was given.
    File(&'a Path),
}

/// Resolve a CLI path argument into a [`LoadSource`].
///
/// Returns [`LoadSource::Stdin`] when `path` is exactly `"-"`,
/// [`LoadSource::File`] for everything else.
fn resolve_source(path: &Path) -> LoadSource<'_> {
    if path == Path::new("-") {
        LoadSource::Stdin
    } else {
        LoadSource::File(path)
    }
}

/// Internal chunk size for bounded stdin reads (8 KiB).
const CHUNK_SIZE: usize = 8 * 1024;

/// Read from a generic [`Read`] source into a `Vec<u8>`, optionally enforcing
/// a byte-count cap.
///
/// When `max_bytes` is `Some(n)`, the function returns an [`io::ErrorKind::InvalidData`]
/// error as soon as the accumulated length would exceed `n`. When `None`, reading
/// continues until EOF with no limit.
#[allow(clippy::indexing_slicing)] // read_size and n are always <= CHUNK_SIZE
fn read_bounded<R: Read>(reader: &mut R, max_bytes: Option<usize>) -> io::Result<Vec<u8>> {
    let mut buf = Vec::new();
    let mut chunk = [0_u8; CHUNK_SIZE];

    loop {
        let read_size = match max_bytes {
            Some(cap) => {
                let remaining = cap.saturating_sub(buf.len());
                if remaining == 0 {
                    // Cap reached — probe for one more byte to distinguish
                    // exact-fit (EOF) from genuine overflow.
                    let mut probe = [0_u8; 1];
                    return if reader.read(&mut probe)? == 0 {
                        Ok(buf)
                    } else {
                        Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            format!("stdin input exceeded {cap} bytes"),
                        ))
                    };
                }
                remaining.min(CHUNK_SIZE)
            }
            None => CHUNK_SIZE,
        };

        let n = reader.read(&mut chunk[..read_size])?;
        if n == 0 {
            break;
        }
        buf.extend_from_slice(&chunk[..n]);
    }

    Ok(buf)
}

/// Load data from a file path, using memory mapping when possible.
///
/// For regular files, this delegates to [`map_file`] for
/// zero-copy access. The path must point to a non-empty, readable file.
///
/// If `path` is `"-"`, stdin is read into a heap buffer with a default
/// 1 GiB limit via [`load_stdin`]. For precise byte-limit control, call
/// [`load_stdin`] directly with the desired cap.
///
/// # Note
///
/// When the caller needs a custom byte limit for stdin input, use
/// `load_stdin(Some(limit))` directly instead of `load("-")`.
///
/// # Errors
///
/// Returns [`io::Error`] if the file cannot be opened, is empty, cannot
/// be memory-mapped, or — when `path` is `"-"` — if stdin input exceeds
/// 1 GiB ([`io::ErrorKind::InvalidData`]).
///
/// # Examples
///
/// ```no_run
/// use mmap_guard::load;
///
/// // Regular file
/// let data = load("input.bin")?;
/// println!("loaded {} bytes", data.len());
///
/// // Stdin via "-" (reads up to 1 GiB)
/// let stdin_data = load("-")?;
/// println!("read {} bytes from stdin", stdin_data.len());
/// # Ok::<(), std::io::Error>(())
/// ```
pub fn load(path: impl AsRef<Path>) -> io::Result<FileData> {
    let path = path.as_ref();
    match resolve_source(path) {
        LoadSource::Stdin => load_stdin(Some(DEFAULT_STDIN_MAX_BYTES)),
        LoadSource::File(p) => map_file(p),
    }
}

/// Read all of stdin into a heap-allocated buffer.
///
/// Returns [`FileData::Loaded`] containing the complete contents of stdin.
/// This is useful for CLI tools that accept piped input via `-`.
///
/// `max_bytes` controls the upper bound on how much data will be read:
/// - `None` — unlimited; reads until EOF.
/// - `Some(n)` — hard cap at `n` bytes; returns an error if exceeded.
///
/// # Errors
///
/// Returns [`io::Error`] if reading from stdin fails, or if `max_bytes`
/// is `Some(n)` and the input exceeds `n` bytes
/// ([`io::ErrorKind::InvalidData`]).
///
/// # Examples
///
/// ```no_run
/// use mmap_guard::load_stdin;
///
/// // Unlimited
/// let data = load_stdin(None)?;
/// println!("read {} bytes from stdin", data.len());
///
/// // Capped at 10 MiB
/// let data = load_stdin(Some(10 * 1024 * 1024))?;
/// # Ok::<(), std::io::Error>(())
/// ```
pub fn load_stdin(max_bytes: Option<usize>) -> io::Result<FileData> {
    let mut stdin = io::stdin();
    let buf = read_bounded(&mut stdin, max_bytes)?;
    Ok(FileData::Loaded(buf))
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use std::io::{self, Cursor, Write};
    use std::path::Path;

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

    // --- read_bounded tests ---

    #[test]
    fn read_bounded_accepts_within_limit() {
        let input = b"hello, world";
        let mut cursor = Cursor::new(input.as_slice());

        let result = read_bounded(&mut cursor, Some(1024)).unwrap();
        assert_eq!(result, input);
    }

    #[test]
    fn read_bounded_returns_error_on_overflow() {
        let input = vec![0xAB_u8; 256];
        let mut cursor = Cursor::new(input.as_slice());

        let err = read_bounded(&mut cursor, Some(100)).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
        assert!(
            err.to_string().contains("100 bytes"),
            "error message should mention the cap: {err}",
        );
    }

    #[test]
    fn read_bounded_unlimited_reads_all() {
        let input = vec![0xCD_u8; 32_000];
        let mut cursor = Cursor::new(input.as_slice());

        let result = read_bounded(&mut cursor, None).unwrap();
        assert_eq!(result.len(), 32_000);
    }

    #[test]
    fn read_bounded_exact_cap_succeeds() {
        let input = b"exactly";
        let mut cursor = Cursor::new(input.as_slice());

        let result = read_bounded(&mut cursor, Some(input.len())).unwrap();
        assert_eq!(result, input);
    }

    #[test]
    fn read_bounded_one_byte_over_cap_fails() {
        let input = b"overflow";
        let mut cursor = Cursor::new(input.as_slice());

        let err = read_bounded(&mut cursor, Some(input.len() - 1)).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
    }

    #[test]
    fn resolve_source_dash_routes_to_stdin() {
        assert_eq!(resolve_source(Path::new("-")), LoadSource::Stdin);
    }

    #[test]
    fn resolve_source_regular_path_routes_to_file() {
        let path = Path::new("/some/file.txt");
        assert_eq!(resolve_source(path), LoadSource::File(path));
    }

    #[test]
    fn resolve_source_dash_prefix_is_not_stdin() {
        let path = Path::new("-extra");
        assert_eq!(resolve_source(path), LoadSource::File(path));
    }

    #[test]
    fn resolve_source_empty_path_is_not_stdin() {
        let path = Path::new("");
        assert_eq!(resolve_source(path), LoadSource::File(path));
    }

    #[test]
    fn dash_routes_to_stdin_via_resolve_source() {
        // Verify that "-" is routed to stdin without performing an actual
        // stdin read (which can block in interactive environments).
        assert_eq!(resolve_source(Path::new("-")), LoadSource::Stdin);
    }

    #[test]
    #[allow(clippy::exit)] // subprocess helper must exit to avoid running the parent path
    fn load_dash_routes_to_stdin() {
        use std::fs;
        use std::process::{Command, Stdio};

        // Subprocess guard: when the env var is set, this process is the
        // child helper. Call load("-"), write the loaded bytes to the file
        // indicated by __MMAP_GUARD_STDIN_OUT, and exit.
        if let Ok(out_path) = std::env::var("__MMAP_GUARD_STDIN_OUT") {
            let result = load("-");
            match result {
                Ok(data) if matches!(data, FileData::Loaded(..)) => {
                    fs::write(&out_path, &*data).unwrap();
                    std::process::exit(0);
                }
                _ => std::process::exit(1),
            }
        }

        let current_exe = std::env::current_exe().expect("failed to get current exe");
        let payload = b"hello from stdin";

        // Temp file the child will write its loaded bytes to.
        let out_file = NamedTempFile::new().expect("failed to create output temp file");
        let out_path = out_file.path().to_owned();

        let mut child = Command::new(&current_exe)
            .env("__MMAP_GUARD_STDIN_OUT", &out_path)
            .arg("--exact")
            .arg("load::tests::load_dash_routes_to_stdin")
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .expect("failed to spawn child");

        // Write payload then close stdin so the child sees EOF.
        {
            let child_stdin = child.stdin.as_mut().expect("missing child stdin");
            child_stdin
                .write_all(payload)
                .expect("failed to write to child stdin");
        }
        drop(child.stdin.take());

        let output = child.wait_with_output().expect("failed to wait on child");
        assert!(
            output.status.success(),
            "child exited with failure: {}\nstderr: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr),
        );

        let written = fs::read(&out_path).expect("failed to read child output file");
        assert_eq!(
            written, payload,
            "child output did not match expected payload"
        );
    }

    #[test]
    fn read_bounded_produces_loaded_variant() {
        // Validate the bounded-read path that load("-") delegates to,
        // using a Cursor to avoid dependence on real process stdin.
        let input = b"simulated stdin";
        let mut cursor = Cursor::new(input.as_slice());

        let buf = read_bounded(&mut cursor, Some(DEFAULT_STDIN_MAX_BYTES)).unwrap();
        let data = FileData::Loaded(buf);

        assert!(
            matches!(data, FileData::Loaded(..)),
            "expected Loaded variant, got {data:?}"
        );
        assert_eq!(&*data, input);
    }
}
