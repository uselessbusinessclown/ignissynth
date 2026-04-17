#![no_main]
//! Fuzz target: `decode_form`.
//!
//! Contract under test:
//!
//! 1. `decode_form` must never panic on arbitrary input bytes. Any
//!    malformed input must be reported as `Err(WireError::...)`.
//! 2. If `decode_form(b)` returns `Ok(form)`, then
//!    `encode_form(&form)` must succeed and produce bytes whose
//!    re-decode equals the original `form`. (The first encoding may
//!    differ from `b` byte-for-byte if `b` was not in canonical form;
//!    we verify the canonical round-trip rather than identity.)
//!
//! Run locally:
//!
//!     cd ignis0
//!     cargo +nightly fuzz run decode_form -- -runs=1000

use libfuzzer_sys::fuzz_target;

use ignis0::{decode_form, encode_form};

fuzz_target!(|data: &[u8]| {
    if let Ok(form) = decode_form(data) {
        // Round-trip canonical encoding.
        let encoded = encode_form(&form)
            .expect("encode_form must succeed on a Form produced by decode_form");
        let redecoded = decode_form(&encoded)
            .expect("decode_form must succeed on bytes produced by encode_form");
        assert_eq!(form, redecoded, "encode/decode round-trip must be stable");
    }
});
