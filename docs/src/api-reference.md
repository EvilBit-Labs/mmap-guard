# API Reference

Full rustdoc is available at [docs.rs/mmap-guard](https://docs.rs/mmap-guard) and in the [API docs](../api/mmap_guard/index.html) section of this book.

## Public API Summary

### `FileData` (enum)

```rust,ignore
pub enum FileData {
    Mapped(Mmap, File), // File retains the advisory lock
    Loaded(Vec<u8>),
}
```

Implements:

- `Deref<Target = [u8]>` — dereferences to a byte slice
- `AsRef<[u8]>` — converts to a byte slice reference
- `Debug` — debug formatting

### `map_file`

```rust,ignore
pub fn map_file(path: impl AsRef<Path>) -> io::Result<FileData>
```

Opens a file, verifies it is non-empty, and creates a read-only memory mapping. Returns `FileData::Mapped` on success.

**Errors:**

| Condition                               | `io::ErrorKind`    |
| --------------------------------------- | ------------------ |
| File not found                          | `NotFound`         |
| Permission denied                       | `PermissionDenied` |
| File is empty                           | `InvalidInput`     |
| Another process holds an exclusive lock | `WouldBlock`       |
| Mapping fails                           | (OS-specific)      |

### `load`

```rust,ignore
pub fn load(path: impl AsRef<Path>) -> io::Result<FileData>
```

Loads data from a file path using memory mapping. If `path` is `"-"`, delegates to `load_stdin(Some(1_073_741_824))` (1 GiB cap) and returns `FileData::Loaded`. All other paths delegate to `map_file`.

**Note:** For callers that need precise stdin control (custom cap or no cap), call `load_stdin(max_bytes)` directly rather than relying on the `"-"` shortcut.

### `load_stdin`

```rust,ignore
pub fn load_stdin(max_bytes: Option<usize>) -> io::Result<FileData>
```

Reads stdin in bounded chunks into a heap-allocated buffer. If `max_bytes` is `Some(n)`, returns an `InvalidData` error if stdin exceeds `n` bytes (no partial data returned). `None` reads to EOF with no limit. Returns `FileData::Loaded`.
