#![no_main]

use std::io::Write;

use libfuzzer_sys::fuzz_target;
use mmap_guard::map_file;
use tempfile::NamedTempFile;

fuzz_target!(|data: &[u8]| {
    // Write fuzzer-provided bytes to a temp file, then map it.
    let mut tmp = match NamedTempFile::new() {
        Ok(f) => f,
        // If temp file creation fails, the environment is broken and
        // the entire fuzz run is useless — panic to surface the issue.
        Err(e) => panic!("temp dir is broken — fuzzing is impossible: {e}"),
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
            // Empty files cannot be mapped — the only error expected in this
            // context (no lock contention on fresh temp files). Any other error
            // indicates an environmental issue worth investigating.
            assert_eq!(
                e.kind(),
                std::io::ErrorKind::InvalidInput,
                "unexpected error kind: {e}",
            );
            assert!(data.is_empty(), "got InvalidInput but data was non-empty");
        }
    }
});
