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

Loads data from a file path using memory mapping. Currently delegates directly to `map_file`.

### `load_stdin`

```rust,ignore
pub fn load_stdin() -> io::Result<FileData>
```

Reads all of stdin into a heap-allocated buffer. Returns `FileData::Loaded`.
