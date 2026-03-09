# Testing

## Running Tests

```bash
# Run all tests with nextest (preferred)
just test

# Run a single test
cargo nextest run map_file_reads_content

# Run with standard cargo test (includes doctests)
cargo test

# Run all tests including ignored/slow tests
just test-all
```

## Test Organization

Tests are co-located with their source modules using `#[cfg(test)]` blocks:

| Module         | Tests                                                                                                            |
| -------------- | ---------------------------------------------------------------------------------------------------------------- |
| `file_data.rs` | `Deref`/`AsRef` impls, empty variant                                                                             |
| `map.rs`       | Successful mapping, empty file rejection, missing file                                                           |
| `load.rs`      | File loading via mmap, stdin handling with byte caps, path resolution (`"-"` routing), empty/missing file errors |

### Clippy in Tests

The crate denies `unwrap_used` and warns on `expect_used` globally. Test modules annotate with:

```rust,ignore
#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    // ...
}
```

## Testing stdin Functionality

The `load("-")` function reads from real process stdin and **must not be called in unit tests** where stdin is controlled by the test harness. Directly invoking `load("-")` in a unit test may block indefinitely or behave inconsistently across test runners.

### Unit Testing

Unit tests for stdin logic should use the internal `read_bounded` function with `Cursor<Vec<u8>>` to test data processing and byte-cap enforcement:

```rust,ignore
use std::io::Cursor;
let mut cursor = Cursor::new(b"test input");
let result = read_bounded(&mut cursor, Some(1024)).unwrap();
```

Unit tests for path resolution should use `resolve_source` to verify that `"-"` is correctly routed to stdin logic:

```rust,ignore
assert_eq!(resolve_source(Path::new("-")), LoadSource::Stdin);
```

### Integration Testing

Integration tests for `load("-")` must spawn the test binary as a subprocess with piped stdin and an environment variable guard to prevent accidental execution in the parent process:

```rust,ignore
// Set __MMAP_GUARD_STDIN_OUT env var in child, write result to temp file
let mut child = Command::new(&current_exe)
    .env("__MMAP_GUARD_STDIN_OUT", &out_path)
    .stdin(Stdio::piped())
    .spawn()
    .expect("failed to spawn child");
```

Subprocess integration tests should use a temporary file (not stdout) for child-to-parent data transfer, since the test harness may write to stdout during test execution.

## Coverage

```bash
# Generate LCOV report
just coverage

# Check against 85% threshold (used in CI)
just coverage-check

# Open interactive HTML report
just coverage-report

# Print summary by file
just coverage-summary
```

Coverage reports exclude test code and focus on `src/` via the Codecov configuration.

## Writing New Tests

When adding tests, follow these patterns:

1. Use `tempfile::NamedTempFile` for tests that need real files on disk
2. Test both success and error paths
3. Assert specific `io::ErrorKind` values for error cases:
   - `InvalidInput` for empty files
   - `InvalidData` when stdin exceeds `max_bytes` limit in `load_stdin`
   - `WouldBlock` for advisory lock contention from `map_file`
4. Check the correct `FileData` variant is returned (`Mapped` vs `Loaded`)

### Testing Lock Contention

Testing lock contention requires spawning a subprocess to hold an exclusive lock. This is necessary because `flock()` locks don't conflict within the same process on macOS — locks are per open-file-description, and different file descriptors in the same process do not contend.

Use a subprocess lock holder (e.g., `python3 -c "import fcntl; ..."`) to acquire an exclusive lock, then verify that `map_file` returns `WouldBlock`:

```rust,ignore
let mut child = Command::new("python3")
    .arg("-c")
    .arg("import fcntl, os, sys; \
          fd = os.open(sys.argv[1], os.O_RDONLY); \
          fcntl.flock(fd, fcntl.LOCK_EX); \
          sys.stdout.write('locked\\n'); sys.stdout.flush(); \
          sys.stdin.readline()")
    .arg(&path)
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .spawn()
    .expect("failed to spawn lock holder");
```
