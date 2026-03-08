#![no_main]

use std::io::Write;

use libfuzzer_sys::fuzz_target;
use mmap_guard::map_file;
use tempfile::NamedTempFile;

fuzz_target!(|data: &[u8]| {
    // Write fuzzer-provided bytes to a temp file, then map it.
    let mut tmp = match NamedTempFile::new() {
        Ok(f) => f,
        Err(_) => return, // OS resource exhaustion — skip this input
    };

    if tmp.write_all(data).is_err() || tmp.flush().is_err() {
        return; // I/O failure on temp file — not interesting
    }

    match map_file(tmp.path()) {
        Ok(mapped) => {
            // Round-trip integrity: mapped bytes must equal what we wrote.
            assert_eq!(&*mapped, data, "round-trip mismatch");
        }
        Err(e) => {
            // Empty files cannot be mapped — this is the only expected error.
            assert_eq!(
                e.kind(),
                std::io::ErrorKind::InvalidInput,
                "unexpected error kind: {e}",
            );
            assert!(data.is_empty(), "got InvalidInput but data was non-empty");
        }
    }
});
