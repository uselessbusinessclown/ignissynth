#![no_main]
//! Fuzz target: `parse_form_lines`.
//!
//! Contract under test:
//!
//! `parse_form_lines` must never panic on arbitrary UTF-8 input. Any
//! malformed input must be reported as `Err(ParseError::...)`. This is
//! the line-oriented scaffold parser; even though it is *pre-wire*, it
//! still feeds `PARSEFORM` and the `S-03.read` path eventually, so a
//! panic here would abort the host process during ignition.
//!
//! Run locally:
//!
//!     cd ignis0
//!     cargo +nightly fuzz run parse_form_lines -- -runs=1000

use libfuzzer_sys::fuzz_target;

use ignis0::parse_form_lines;

fuzz_target!(|data: &[u8]| {
    // The parser takes &str; only fuzz valid UTF-8. libFuzzer will
    // still explore a wide UTF-8 keyspace including null bytes,
    // multi-byte sequences, and weird whitespace.
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = parse_form_lines(s);
    }
});
