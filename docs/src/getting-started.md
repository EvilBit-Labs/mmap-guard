# Getting Started

## Installation

Add `mmap-guard` to your `Cargo.toml`:

```toml
[dependencies]
mmap-guard = "0.1"
```

## Quick Start

### Memory-map a file

```rust,no_run
use mmap_guard::map_file;

fn main() -> std::io::Result<()> {
    let data = map_file("large-file.bin")?;
    println!("file size: {} bytes", data.len());
    println!("first byte: {:#04x}", data[0]);
    Ok(())
}
```

### Accept both files and stdin

```rust,no_run
use mmap_guard::{load, load_stdin};
use std::path::Path;

fn main() -> std::io::Result<()> {
    let path = std::env::args().nth(1).unwrap_or_else(|| "-".into());

    let data = if path == "-" {
        load_stdin()?
    } else {
        load(&path)?
    };

    println!("loaded {} bytes", data.len());
    // data derefs to &[u8] — use it like any byte slice
    Ok(())
}
```

## The `FileData` Type

[`FileData`](https://docs.rs/mmap-guard/latest/mmap_guard/enum.FileData.html) is an enum with two variants:

- **`Mapped`** — zero-copy memory-mapped data
- **`Loaded`** — heap-allocated buffer (used for stdin/pipes)

Both variants implement `Deref<Target = [u8]>` and `AsRef<[u8]>`, so you can use `FileData` anywhere a `&[u8]` is expected without caring which variant is in use.
