//! Property tests for `map_file` round-trip integrity.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::io::Write;

use proptest::prelude::*;
use tempfile::NamedTempFile;

use mmap_guard::{FileData, map_file};

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// Writing arbitrary bytes to a temp file and mapping it back must
    /// produce identical contents (round-trip integrity).
    #[test]
    fn prop_map_file_roundtrip(data in proptest::collection::vec(any::<u8>(), 1..65_536)) {
        let mut tmp = NamedTempFile::new().expect("failed to create temp file");
        tmp.write_all(&data).expect("failed to write");
        tmp.flush().expect("failed to flush");

        let mapped = map_file(tmp.path()).expect("map_file failed");
        assert!(matches!(mapped, FileData::Mapped(..)), "expected Mapped variant");
        prop_assert_eq!(&*mapped, &data[..]);
    }

}

/// Empty files must be rejected with `InvalidInput`.
#[test]
fn map_file_rejects_empty() {
    let tmp = NamedTempFile::new().expect("failed to create temp file");
    let err = map_file(tmp.path()).expect_err("expected error for empty file");
    assert_eq!(err.kind(), std::io::ErrorKind::InvalidInput);
}
