//! Golden tests for canonical hand-encoded Forms in `ignis0`.
//!
//! ## Scope (issue #15, narrow reading)
//!
//! IgnisSynth's discipline draws a hard line between **prose
//! `.form` specifications** (in `kernel/forms/`) and **wire-form
//! bytes** (what an external build process produces by sealing
//! those specifications). The prose specs are s-expressions; the
//! line-oriented `parser::parse_form_lines` in this crate cannot
//! read them. There is therefore no honest way today to write a
//! "for each `.form` file" golden test without violating the
//! prose-vs-bytes invariant called out in `src/wire.rs`:
//!
//! > "No files under `kernel/forms/` are touched or consulted.
//! >  Those files are IgnisSynth prose specifications, not
//! >  wire-form bytes; the wire codec's contract is 'bytes in,
//! >  bytes out, hash matches', and exercising it against prose
//! >  would be category error."
//!
//! What this file pins instead is the **wire encoding and BLAKE3
//! hash of the canonical Forms that already live in `ignis0` and
//! are hand-encoded against the IL**:
//!
//! 1. **F** — the canonical fixed-point Form from
//!    `kernel/IGNITION-BOOTSTRAP.md` § Step 2 (`STORE 0; LOAD 0;
//!    PUSH 1; ADD; RET`). Used by the A9.3 fixed-point check.
//! 2. **micro-S-07/execute (level 1)** — the minimal stand-in for
//!    the real `S-07/execute` Form, used by the A9.3 indirect
//!    case. `STORE 0; LOAD 0; CALL F 1; RET`.
//!
//! These are the **only** Forms `ignis0` constructs end-to-end
//! today. The full primary-Form golden coverage will become
//! possible once `kernel/forms/helpers/parser.form` is loadable
//! and an external build process can turn the prose `.form` files
//! into canonical wire bytes (post-`v0.5.0-build`). When that
//! happens, this file should be extended with one golden per
//! shipped Form, keyed by file path.
//!
//! ## What each assertion catches
//!
//! - `F_GOLDEN_BYTES` mismatch — wire codec, opcode tags, trap-kind
//!   tags, or canonical ULEB128 encoding drifted.
//! - `F_GOLDEN_HASH_HEX` mismatch — bytes mismatch (above) or a
//!   blake3 dependency surprise.
//! - `MICRO_S07_GOLDEN_BYTES` mismatch — either `CALL`'s wire
//!   encoding (form-hash + n ULEB128) drifted, or F's hash drifted
//!   upstream and the CALL target moved with it.
//! - Pretty-print mismatch — `Opcode → text` rendering changed.
//!   This is the human-facing side of the same codec invariant.
//!
//! ## Regenerating goldens
//!
//! Run `cargo test --test golden -- --ignored update_goldens`.
//! That ignored test prints `pub const` declarations to stdout
//! that you can paste into the constants section below. Any
//! regeneration MUST be accompanied by a commit message that
//! explains **why** the canonical bytes changed — silent codec
//! drift is exactly what these goldens exist to catch.
//!
//! A9.4 reminder: these Forms only exist inside the stage-0
//! substrate's test binary. No stage-0 artifact crosses into the
//! habitat.

use ignis0::{
    decode_form, encode_form, pretty_print_with_header, Form as WireForm, Opcode, SubstanceHash,
    TrapKind, Value,
};

// ── Goldens ────────────────────────────────────────────────────

/// Canonical bytes of the F fixed-point Form, including the
/// trailing 32-byte BLAKE3 self-hash. 64 bytes total: 32-byte
/// prefix (magic, version, type tag, arity, locals_n, declared
/// caps + traps, code) followed by its self-hash.
#[rustfmt::skip]
const F_GOLDEN_BYTES: &[u8] = &[
    0x49, 0x53, 0x46, 0x31, 0x01, 0x07, 0x46, 0x6f, 0x72, 0x6d, 0x2f, 0x76,
    0x31, 0x01, 0x01, 0x00, 0x01, 0x01, 0x03, 0x4e, 0x61, 0x74, 0x05, 0x03,
    0x00, 0x02, 0x00, 0x00, 0x02, 0x01, 0x04, 0x0b, 0xf3, 0x12, 0x81, 0xd3,
    0x0a, 0x8e, 0x13, 0xfa, 0xd2, 0x46, 0x3b, 0x97, 0x0c, 0xb6, 0xf2, 0x20,
    0x63, 0x91, 0x89, 0x27, 0x9f, 0x75, 0x76, 0xf3, 0xd1, 0xc1, 0xdc, 0x1e,
    0x42, 0x6a, 0x49, 0xfb,
];

/// BLAKE3 of `F_GOLDEN_BYTES[..len-32]`. Pinned so that any change
/// to the canonical encoding is loud, not silent.
const F_GOLDEN_HASH_HEX: &str = "f31281d30a8e13fad2463b970cb6f220639189279f7576f3d1c1dc1e426a49fb";

/// Canonical bytes of the level-1 micro-S-07/execute wrapper. 94
/// bytes total. The CALL inside this Form references F's hash
/// (`F_GOLDEN_HASH_HEX` bytes), so any change to F propagates here.
#[rustfmt::skip]
const MICRO_S07_GOLDEN_BYTES: &[u8] = &[
    0x49, 0x53, 0x46, 0x31, 0x01, 0x07, 0x46, 0x6f, 0x72, 0x6d, 0x2f, 0x76,
    0x31, 0x01, 0x01, 0x00, 0x01, 0x01, 0x03, 0x4e, 0x61, 0x74, 0x04, 0x03,
    0x00, 0x02, 0x00, 0x0a, 0xf3, 0x12, 0x81, 0xd3, 0x0a, 0x8e, 0x13, 0xfa,
    0xd2, 0x46, 0x3b, 0x97, 0x0c, 0xb6, 0xf2, 0x20, 0x63, 0x91, 0x89, 0x27,
    0x9f, 0x75, 0x76, 0xf3, 0xd1, 0xc1, 0xdc, 0x1e, 0x42, 0x6a, 0x49, 0xfb,
    0x01, 0x0b, 0x22, 0xbd, 0x0a, 0x7d, 0xbf, 0x9b, 0xad, 0xbe, 0xe2, 0x1d,
    0x7e, 0xf0, 0x71, 0x53, 0xb4, 0x35, 0xce, 0x1c, 0x64, 0xab, 0x5b, 0x42,
    0xcd, 0xe4, 0xc5, 0xa6, 0xd2, 0x6a, 0x4e, 0x10, 0x04, 0xff,
];

/// BLAKE3 of `MICRO_S07_GOLDEN_BYTES[..len-32]`.
const MICRO_S07_GOLDEN_HASH_HEX: &str =
    "22bd0a7dbf9badbee21d7ef07153b435ce1c64ab5b42cde4c5a6d26a4e1004ff";

// ── Form builders (deliberately re-stated to avoid coupling) ──

/// Build F as a wire-form Form record. Re-stated here rather than
/// imported from `fixed_point.rs` so that an accidental change to
/// the runtime construction can never silently rewrite the golden.
/// If the runtime Form genuinely changes, the test fails loud and
/// the golden must be regenerated with a justifying commit.
fn build_f() -> WireForm {
    WireForm {
        type_tag: "Form/v1".to_string(),
        arity: 1,
        locals_n: 1,
        declared_caps: vec![],
        declared_traps: vec![TrapKind::EType("Nat".into())],
        code: vec![
            Opcode::Store(0),
            Opcode::Load(0),
            Opcode::Push(Value::Nat(1)),
            Opcode::Add,
            Opcode::Ret,
        ],
    }
}

/// Build the level-1 micro-S-07 wrapping `target`. As with `build_f`,
/// re-stated here to keep the golden independent.
fn build_micro_s07(target: SubstanceHash) -> WireForm {
    WireForm {
        type_tag: "Form/v1".to_string(),
        arity: 1,
        locals_n: 1,
        declared_caps: vec![],
        declared_traps: vec![TrapKind::EType("Nat".into())],
        code: vec![
            Opcode::Store(0),
            Opcode::Load(0),
            Opcode::Call { form: target, n: 1 },
            Opcode::Ret,
        ],
    }
}

fn hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{:02x}", b));
    }
    s
}

fn blake3_of(bytes: &[u8]) -> SubstanceHash {
    SubstanceHash(*blake3::hash(bytes).as_bytes())
}

// ── Real tests ────────────────────────────────────────────────

#[test]
fn f_wire_bytes_match_golden() {
    let bytes = encode_form(&build_f()).expect("F encodes");
    assert_eq!(
        bytes, F_GOLDEN_BYTES,
        "F wire bytes drifted. If this was intentional, run \
         `cargo test --test golden -- --ignored update_goldens` \
         and explain WHY in the commit message."
    );
}

#[test]
fn f_hash_matches_golden() {
    // The encoded form's last 32 bytes are the trailing self-hash;
    // we recompute it from the prefix to keep the assertion honest.
    let bytes = encode_form(&build_f()).expect("F encodes");
    let prefix = &bytes[..bytes.len() - 32];
    assert_eq!(
        hex(&blake3_of(prefix).0),
        F_GOLDEN_HASH_HEX,
        "F canonical hash drifted."
    );
}

#[test]
fn f_round_trips() {
    let bytes = encode_form(&build_f()).expect("F encodes");
    let decoded = decode_form(&bytes).expect("F decodes");
    assert_eq!(
        decoded,
        build_f(),
        "decode(encode(F)) should be the identity"
    );
}

#[test]
fn f_pretty_print_stable() {
    let f = build_f();
    let rendered = pretty_print_with_header("F", f.locals_n as usize, &f.code);
    // The pretty-printer is the human-facing side of the same codec
    // invariant. Pin its output so a one-character format change
    // can't slip through unnoticed.
    let expected = "\
; Form: F
; locals-n: 1
; arity: (see ExecState::new call site)

STORE 0
LOAD 0
PUSH 1
ADD
RET
";
    assert_eq!(rendered, expected, "F pretty-print drifted");
}

#[test]
fn micro_s07_wire_bytes_match_golden() {
    // The wrapper's CALL target is F's hash — recompute it from
    // the F golden so we are testing one thing at a time.
    let f_bytes = encode_form(&build_f()).expect("F encodes");
    let f_hash = blake3_of(&f_bytes[..f_bytes.len() - 32]);
    let bytes = encode_form(&build_micro_s07(f_hash)).expect("S07 encodes");
    assert_eq!(
        bytes, MICRO_S07_GOLDEN_BYTES,
        "micro-S-07 wire bytes drifted."
    );
}

#[test]
fn micro_s07_hash_matches_golden() {
    let f_bytes = encode_form(&build_f()).expect("F encodes");
    let f_hash = blake3_of(&f_bytes[..f_bytes.len() - 32]);
    let bytes = encode_form(&build_micro_s07(f_hash)).expect("S07 encodes");
    let prefix = &bytes[..bytes.len() - 32];
    assert_eq!(
        hex(&blake3_of(prefix).0),
        MICRO_S07_GOLDEN_HASH_HEX,
        "micro-S-07 canonical hash drifted."
    );
}

#[test]
fn micro_s07_round_trips() {
    let f_bytes = encode_form(&build_f()).expect("F encodes");
    let f_hash = blake3_of(&f_bytes[..f_bytes.len() - 32]);
    let s07 = build_micro_s07(f_hash);
    let bytes = encode_form(&s07).expect("S07 encodes");
    let decoded = decode_form(&bytes).expect("S07 decodes");
    assert_eq!(decoded, s07, "decode(encode(S07)) should be the identity");
}

// ── Regeneration helper ───────────────────────────────────────

#[test]
#[ignore]
fn update_goldens() {
    // Print pub-const blocks that can be pasted into the constants
    // section above. Run with:
    //
    //     cargo test --test golden -- --ignored update_goldens --nocapture
    let f = build_f();
    let f_bytes = encode_form(&f).expect("F encodes");
    let f_hash = blake3_of(&f_bytes[..f_bytes.len() - 32]);

    println!();
    println!("// --- regenerated F golden ({} bytes) ---", f_bytes.len());
    print!("const F_GOLDEN_BYTES: &[u8] = &[");
    for (i, b) in f_bytes.iter().enumerate() {
        if i % 12 == 0 {
            print!("\n    ");
        }
        print!("0x{:02x}, ", b);
    }
    println!("\n];");
    println!("const F_GOLDEN_HASH_HEX: &str = \"{}\";", hex(&f_hash.0));

    let s07 = build_micro_s07(f_hash);
    let s07_bytes = encode_form(&s07).expect("S07 encodes");
    let s07_hash = blake3_of(&s07_bytes[..s07_bytes.len() - 32]);
    println!();
    println!(
        "// --- regenerated micro-S-07 golden ({} bytes) ---",
        s07_bytes.len()
    );
    print!("const MICRO_S07_GOLDEN_BYTES: &[u8] = &[");
    for (i, b) in s07_bytes.iter().enumerate() {
        if i % 12 == 0 {
            print!("\n    ");
        }
        print!("0x{:02x}, ", b);
    }
    println!("\n];");
    println!(
        "const MICRO_S07_GOLDEN_HASH_HEX: &str = \"{}\";",
        hex(&s07_hash.0)
    );

    let f_pp = pretty_print_with_header("F", f.locals_n as usize, &f.code);
    println!();
    println!("// --- regenerated F pretty-print ---");
    println!("// {:?}", f_pp);
}
