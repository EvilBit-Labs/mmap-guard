# Integration Examples

## Using with `#![forbid(unsafe_code)]` Crates

The primary use case for mmap-guard is enabling memory-mapped I/O in crates that forbid unsafe code.

### Feature-gated mmap support

In your crate's `Cargo.toml`:

```toml
[dependencies]
mmap-guard = { version = "0.1", optional = true }

[features]
mmap = ["dep:mmap-guard"]
```

In your source code:

```rust,ignore
#![forbid(unsafe_code)]

use std::path::Path;
use std::io;

fn load_file(path: &Path) -> io::Result<Vec<u8>> {
    #[cfg(feature = "mmap")]
    {
        let data = mmap_guard::map_file(path)?;
        // FileData derefs to &[u8], but we need owned data
        // if the caller expects Vec<u8>. For zero-copy, pass
        // the FileData directly.
        Ok(data.to_vec())
    }

    #[cfg(not(feature = "mmap"))]
    {
        std::fs::read(path)
    }
}
```

### Zero-copy pipeline

For best performance, pass `FileData` through your pipeline instead of converting to `Vec<u8>`:

```rust,ignore
#![forbid(unsafe_code)]

use mmap_guard::FileData;
use std::path::Path;
use std::io;

fn process_bytes(data: &[u8]) {
    // Works with both Mapped and Loaded variants
    println!("processing {} bytes", data.len());
}

fn run(path: &Path) -> io::Result<()> {
    let data: FileData = mmap_guard::map_file(path)?;
    process_bytes(&data); // zero-copy — no allocation
    Ok(())
}
```

### CLI tool with stdin support

```rust,ignore
use mmap_guard::{load, load_stdin, FileData};
use std::io;

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    let data: FileData = match args.get(1).map(String::as_str) {
        Some("-") | None => load_stdin()?,
        Some(path) => load(path)?,
    };

    // Process data uniformly regardless of source
    println!("{} bytes", data.len());
    Ok(())
}
```
