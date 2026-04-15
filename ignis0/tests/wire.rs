//! Round-trip and negative tests for `ignis0::wire`.
//!
//! The strategy is property-style without pulling in a proptest
//! dependency: a small deterministic PRNG (splitmix64) drives a
//! Form generator that produces varied but well-formed Forms,
//! and every case asserts `decode(encode(f)) == f`.
//!
//! **No file under `kernel/forms/` is read.** Those files are
//! IgnisSynth prose specifications, not wire-form bytes; the
//! wire codec's contract is "bytes in, bytes out, hash matches",
//! and exercising it against prose would be category error.
//!
//! A9.4 guard: every Form constructed here exists only inside
//! the stage-0 substrate's test binary. No stage-0 artifact
//! crosses into the habitat.

use ignis0::wire::{decode_form, encode_form, Form, WireError, MAGIC, VERSION};
use ignis0::{Hash, Opcode, TrapKind, Value};

// ---------- splitmix64 PRNG ----------

struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Self {
        Rng(seed)
    }
    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E3779B97F4A7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
        z ^ (z >> 31)
    }
    fn gen_u32(&mut self, max: u32) -> u32 {
        if max == 0 {
            0
        } else {
            (self.next_u64() % max as u64) as u32
        }
    }
    fn gen_i32(&mut self) -> i32 {
        self.next_u64() as i32
    }
    fn gen_bool(&mut self) -> bool {
        self.next_u64() & 1 == 1
    }
    fn gen_hash(&mut self) -> Hash {
        let mut out = [0u8; 32];
        for chunk in out.chunks_mut(8) {
            let w = self.next_u64().to_le_bytes();
            chunk.copy_from_slice(&w);
        }
        Hash(out)
    }
    fn gen_string(&mut self, max_len: u32) -> String {
        let n = self.gen_u32(max_len);
        let mut s = String::with_capacity(n as usize);
        for _ in 0..n {
            // ASCII letters only to avoid UTF-8 corner cases; the
            // codec handles full UTF-8 but the generator doesn't
            // need to.
            let c = (b'a' + (self.next_u64() % 26) as u8) as char;
            s.push(c);
        }
        s
    }
}

// ---------- generators ----------

fn gen_trapkind(rng: &mut Rng) -> TrapKind {
    match rng.gen_u32(11) {
        0 => TrapKind::EBadLocal(rng.gen_string(8)),
        1 => TrapKind::EType(rng.gen_string(8)),
        2 => TrapKind::EUnderflow,
        3 => TrapKind::EUnheld(rng.gen_string(8)),
        4 => TrapKind::ENotHeld,
        5 => TrapKind::EStale,
        6 => TrapKind::EOverBudget,
        7 => TrapKind::EAssert,
        8 => TrapKind::EUnauthorised,
        9 => TrapKind::EIgnited,
        _ => TrapKind::EReplayDiverged,
    }
}

fn gen_value(rng: &mut Rng) -> Value {
    match rng.gen_u32(7) {
        0 => Value::Unit,
        1 => Value::Bool(rng.gen_bool()),
        2 => Value::Nat(rng.next_u64() as u128 ^ ((rng.next_u64() as u128) << 64)),
        3 => Value::Hash(rng.gen_hash()),
        4 => {
            let n = rng.gen_u32(16) as usize;
            let mut b = vec![0u8; n];
            for byte in &mut b {
                *byte = rng.next_u64() as u8;
            }
            Value::Bytes(b)
        }
        5 => Value::Cell(rng.gen_hash()),
        _ => Value::Cont(rng.gen_hash()),
    }
}

fn gen_opcode(rng: &mut Rng) -> Opcode {
    match rng.gen_u32(35) {
        0 => Opcode::Push(gen_value(rng)),
        1 => Opcode::Pop,
        2 => Opcode::Load(rng.gen_u32(32)),
        3 => Opcode::Store(rng.gen_u32(32)),
        4 => Opcode::Add,
        5 => Opcode::Sub,
        6 => Opcode::Eq,
        7 => Opcode::Lt,
        8 => Opcode::Jmp(rng.gen_i32()),
        9 => Opcode::Jmpz(rng.gen_i32()),
        10 => Opcode::Call {
            form: rng.gen_hash(),
            n: rng.gen_u32(8),
        },
        11 => Opcode::Ret,
        12 => Opcode::MakePair,
        13 => Opcode::Fst,
        14 => Opcode::Snd,
        15 => Opcode::MakeVec(rng.gen_u32(8)),
        16 => Opcode::Seal(rng.gen_string(12)),
        17 => Opcode::Read,
        18 => Opcode::Pin,
        19 => Opcode::Unpin,
        20 => Opcode::CapHeld,
        21 => Opcode::Attenuate,
        22 => Opcode::Invoke { n: rng.gen_u32(8) },
        23 => Opcode::Revoke,
        24 => Opcode::Append,
        25 => Opcode::Why,
        26 => Opcode::Yield,
        27 => Opcode::Split,
        28 => Opcode::Trap(gen_trapkind(rng)),
        29 => Opcode::Assert,
        30 => Opcode::SelfHash,
        31 => Opcode::ParseForm,
        32 => Opcode::BindSlot,
        33 => Opcode::ReadSlot,
        _ => Opcode::CallI { n: rng.gen_u32(8) },
    }
}

fn gen_form(rng: &mut Rng, max_code: u32) -> Form {
    let caps_n = rng.gen_u32(4);
    let declared_caps = (0..caps_n).map(|_| rng.gen_hash()).collect();
    let traps_n = rng.gen_u32(4);
    let declared_traps = (0..traps_n).map(|_| gen_trapkind(rng)).collect();
    let code_n = rng.gen_u32(max_code).max(1);
    let code = (0..code_n).map(|_| gen_opcode(rng)).collect();
    Form {
        type_tag: "Form/v1".to_string(),
        arity: rng.gen_u32(8),
        locals_n: rng.gen_u32(32),
        declared_caps,
        declared_traps,
        code,
    }
}

// ---------- round-trip property tests ----------

#[test]
fn roundtrip_1024_random_forms() {
    let mut rng = Rng::new(0xC0FFEE);
    for seed in 0..1024u64 {
        let mut local = Rng::new(seed.wrapping_mul(rng.next_u64() | 1));
        let form = gen_form(&mut local, 24);
        let bytes = match encode_form(&form) {
            Ok(b) => b,
            Err(e) => panic!("encode failed on seed {}: {:?}", seed, e),
        };
        let decoded = match decode_form(&bytes) {
            Ok(d) => d,
            Err(e) => panic!("decode failed on seed {}: {:?}", seed, e),
        };
        assert_eq!(decoded, form, "round-trip mismatch on seed {}", seed);
    }
}

#[test]
fn roundtrip_large_forms() {
    // Exercise longer code vectors; LEB128 crossing byte boundaries.
    let mut rng = Rng::new(0xDEADBEEF);
    for _ in 0..16 {
        let form = gen_form(&mut rng, 256);
        let bytes = encode_form(&form).unwrap();
        let decoded = decode_form(&bytes).unwrap();
        assert_eq!(decoded, form);
    }
}

#[test]
fn canonical_bytes_are_deterministic() {
    // Two separate encodes of the same Form must produce
    // byte-identical outputs. This pins down the "canonicality"
    // clause in IL.md § Canonicality.
    let form = Form {
        type_tag: "Form/v1".into(),
        arity: 1,
        locals_n: 1,
        declared_caps: vec![Hash([7u8; 32])],
        declared_traps: vec![TrapKind::EType("Nat".into())],
        code: vec![
            Opcode::Store(0),
            Opcode::Load(0),
            Opcode::Push(Value::Nat(1)),
            Opcode::Add,
            Opcode::Ret,
        ],
    };
    let a = encode_form(&form).unwrap();
    let b = encode_form(&form).unwrap();
    assert_eq!(a, b);
}

#[test]
fn magic_and_version_present() {
    let form = Form {
        type_tag: "Form/v1".into(),
        arity: 0,
        locals_n: 0,
        declared_caps: vec![],
        declared_traps: vec![],
        code: vec![Opcode::Ret],
    };
    let bytes = encode_form(&form).unwrap();
    assert_eq!(&bytes[..4], MAGIC);
    assert_eq!(bytes[4], VERSION);
    // Trailing 32 bytes are the hash of the prefix.
    let body_end = bytes.len() - 32;
    let expected = blake3::hash(&bytes[..body_end]);
    assert_eq!(&bytes[body_end..], expected.as_bytes());
}

// ---------- negative tests ----------

#[test]
fn truncated_input_rejected() {
    let form = Form {
        type_tag: "Form/v1".into(),
        arity: 0,
        locals_n: 0,
        declared_caps: vec![],
        declared_traps: vec![],
        code: vec![Opcode::Ret],
    };
    let bytes = encode_form(&form).unwrap();
    for cut in 0..bytes.len() {
        let short = &bytes[..cut];
        assert!(decode_form(short).is_err(), "truncation at {}", cut);
    }
}

#[test]
fn bad_opcode_tag_rejected() {
    // Build a minimal valid Form and then corrupt the single
    // code byte to an unassigned tag (0x23..=0xFF are reserved
    // after the CALLI addition).
    let form = Form {
        type_tag: "Form/v1".into(),
        arity: 0,
        locals_n: 0,
        declared_caps: vec![],
        declared_traps: vec![],
        code: vec![Opcode::Ret],
    };
    let mut bytes = encode_form(&form).unwrap();
    // Find the RET byte: it is the last byte before the trailing
    // 32-byte hash.
    let body_end = bytes.len() - 32;
    bytes[body_end - 1] = 0xFE;
    // Re-seal the trailing hash so we get past BadFormHash and
    // actually exercise BadOpcodeTag.
    let h = blake3::hash(&bytes[..body_end]);
    bytes[body_end..].copy_from_slice(h.as_bytes());
    assert!(decode_form(&bytes).is_err());
}

#[test]
fn tampered_hash_rejected() {
    let form = Form {
        type_tag: "Form/v1".into(),
        arity: 0,
        locals_n: 0,
        declared_caps: vec![],
        declared_traps: vec![],
        code: vec![Opcode::Ret],
    };
    let mut bytes = encode_form(&form).unwrap();
    let last = bytes.len() - 1;
    bytes[last] ^= 0x01;
    assert_eq!(decode_form(&bytes).unwrap_err(), WireError::BadFormHash);
}

#[test]
fn trailing_garbage_rejected() {
    let form = Form {
        type_tag: "Form/v1".into(),
        arity: 0,
        locals_n: 0,
        declared_caps: vec![],
        declared_traps: vec![],
        code: vec![Opcode::Ret],
    };
    let mut bytes = encode_form(&form).unwrap();
    bytes.push(0x00);
    // Pushing a byte breaks the trailing-hash invariant first,
    // so we resync the hash to isolate the trailing-garbage case.
    let body_end = bytes.len() - 32;
    let h = blake3::hash(&bytes[..body_end]);
    bytes[body_end..].copy_from_slice(h.as_bytes());
    let err = decode_form(&bytes).unwrap_err();
    // After the resync, the extra byte is now inside the hashed
    // prefix, which means the decoder either sees it as a stray
    // opcode (BadOpcodeTag) or as trailing garbage depending on
    // where it lands. We accept either: both mean "rejected".
    assert!(matches!(
        err,
        WireError::TrailingGarbage | WireError::Truncated
    ));
}

#[test]
fn every_opcode_tag_roundtrips_individually() {
    // Stronger than the random generator: exercise each variant
    // at least once with a known payload.
    let cases: Vec<Opcode> = vec![
        Opcode::Push(Value::Unit),
        Opcode::Push(Value::Bool(true)),
        Opcode::Push(Value::Bool(false)),
        Opcode::Push(Value::Nat(0)),
        Opcode::Push(Value::Nat(u128::MAX)),
        Opcode::Push(Value::Hash(Hash([3u8; 32]))),
        Opcode::Push(Value::Bytes(vec![1, 2, 3, 4, 5])),
        Opcode::Push(Value::Cell(Hash([4u8; 32]))),
        Opcode::Push(Value::Cont(Hash([5u8; 32]))),
        Opcode::Pop,
        Opcode::Load(0),
        Opcode::Load(u32::MAX),
        Opcode::Store(0),
        Opcode::Store(u32::MAX),
        Opcode::Add,
        Opcode::Sub,
        Opcode::Eq,
        Opcode::Lt,
        Opcode::Jmp(0),
        Opcode::Jmp(i32::MAX),
        Opcode::Jmp(i32::MIN),
        Opcode::Jmpz(-1),
        Opcode::Call {
            form: Hash([9u8; 32]),
            n: 0,
        },
        Opcode::Call {
            form: Hash([9u8; 32]),
            n: 42,
        },
        Opcode::CallI { n: 0 },
        Opcode::CallI { n: 3 },
        Opcode::CallI { n: u32::MAX },
        Opcode::Ret,
        Opcode::MakePair,
        Opcode::Fst,
        Opcode::Snd,
        Opcode::MakeVec(0),
        Opcode::MakeVec(u32::MAX),
        Opcode::Seal("".into()),
        Opcode::Seal("substance/v1".into()),
        Opcode::Read,
        Opcode::Pin,
        Opcode::Unpin,
        Opcode::CapHeld,
        Opcode::Attenuate,
        Opcode::Invoke { n: 0 },
        Opcode::Invoke { n: 3 },
        Opcode::Invoke { n: u32::MAX },
        Opcode::Revoke,
        Opcode::Append,
        Opcode::Why,
        Opcode::Yield,
        Opcode::Split,
        Opcode::Trap(TrapKind::EUnderflow),
        Opcode::Trap(TrapKind::EType("ctx".into())),
        Opcode::Trap(TrapKind::EBadLocal("locals[3]".into())),
        Opcode::Trap(TrapKind::EUnheld("cap".into())),
        Opcode::Trap(TrapKind::ENotHeld),
        Opcode::Trap(TrapKind::EStale),
        Opcode::Trap(TrapKind::EOverBudget),
        Opcode::Trap(TrapKind::EAssert),
        Opcode::Trap(TrapKind::EUnauthorised),
        Opcode::Trap(TrapKind::EIgnited),
        Opcode::Trap(TrapKind::EReplayDiverged),
        Opcode::Assert,
        Opcode::SelfHash,
        Opcode::ParseForm,
        Opcode::BindSlot,
        Opcode::ReadSlot,
    ];
    for op in cases {
        let form = Form {
            type_tag: "Form/v1".into(),
            arity: 0,
            locals_n: 0,
            declared_caps: vec![],
            declared_traps: vec![],
            code: vec![op.clone()],
        };
        let bytes = encode_form(&form).unwrap();
        let decoded = decode_form(&bytes).unwrap();
        assert_eq!(decoded, form, "opcode round-trip failed for {:?}", op);
    }
}

#[test]
fn calli_tag_is_0x22() {
    // Pin the tag byte so a future rewrite of the wire codec
    // cannot silently remap CALLI onto another opcode's slot.
    let form = Form {
        type_tag: "Form/v1".into(),
        arity: 0,
        locals_n: 0,
        declared_caps: vec![],
        declared_traps: vec![],
        code: vec![Opcode::CallI { n: 0 }],
    };
    let bytes = encode_form(&form).unwrap();
    // Trailing 32 bytes are the hash; the byte before the
    // ULEB128 `n=0` is the opcode tag.
    let body_end = bytes.len() - 32;
    // body: ...(code header)... 0x22 0x00 (hash)
    // The 0x22 tag is two bytes before body_end (tag + 1-byte ULEB128 zero).
    assert_eq!(bytes[body_end - 2], 0x22, "CALLI must encode as tag 0x22");
    assert_eq!(
        bytes[body_end - 1],
        0x00,
        "CALLI n=0 must ULEB128-encode as a single 0x00 byte"
    );
}

#[test]
fn scaffold_trap_cannot_enter_wire_form() {
    let form = Form {
        type_tag: "Form/v1".into(),
        arity: 0,
        locals_n: 0,
        declared_caps: vec![],
        declared_traps: vec![],
        code: vec![Opcode::Trap(TrapKind::NotImplemented("stub".into()))],
    };
    assert_eq!(
        encode_form(&form).unwrap_err(),
        WireError::ScaffoldTrapInWireForm
    );
}
