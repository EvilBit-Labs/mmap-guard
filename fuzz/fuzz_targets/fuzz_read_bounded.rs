#![no_main]

use std::io::Cursor;

use libfuzzer_sys::arbitrary::{self, Arbitrary};
use libfuzzer_sys::fuzz_target;
use mmap_guard::read_bounded;

/// Structured fuzz input — the fuzzer controls both the data and the cap.
/// `cap_raw` is `u16` to keep the search space small while still covering
/// the interesting boundary conditions (0, 1, exact-fit, overflow).
#[derive(Arbitrary, Debug)]
struct ReadBoundedInput {
    cap_raw: Option<u16>,
    data: Vec<u8>,
}

fuzz_target!(|input: ReadBoundedInput| {
    let cap = input.cap_raw.map(usize::from);
    let mut cursor = Cursor::new(&input.data);

    match read_bounded(&mut cursor, cap) {
        Ok(buf) => {
            // Length must not exceed the cap (when set).
            if let Some(c) = cap {
                assert!(buf.len() <= c, "buf.len() {} exceeded cap {c}", buf.len());
            }
            // Contents must match the input prefix.
            assert_eq!(&buf[..], &input.data[..buf.len()]);
        }
        Err(e) => {
            // The only expected error is overflow (InvalidData).
            assert_eq!(e.kind(), std::io::ErrorKind::InvalidData);
            // An overflow error should only occur when the input is genuinely
            // longer than the cap.
            if let Some(c) = cap {
                assert!(
                    input.data.len() > c,
                    "got InvalidData but data.len() {} <= cap {c}",
                    input.data.len(),
                );
            }
        }
    }
});
