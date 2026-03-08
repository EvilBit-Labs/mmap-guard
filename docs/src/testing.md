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

| Module         | Tests                                                  |
| -------------- | ------------------------------------------------------ |
| `file_data.rs` | `Deref`/`AsRef` impls, empty variant                   |
| `map.rs`       | Successful mapping, empty file rejection, missing file |
| `load.rs`      | File loading via mmap, empty/missing file errors       |

### Clippy in Tests

The crate denies `unwrap_used` and warns on `expect_used` globally. Test modules annotate with:

```rust,ignore
#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    // ...
}
```

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
3. Assert specific `io::ErrorKind` values for error cases
4. Check the correct `FileData` variant is returned (`Mapped` vs `Loaded`)
