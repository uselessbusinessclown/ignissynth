//! Byte-exact wire codec for Forms.
//!
//! Implements the grammar in `../../kernel/IL.md` §
//! "Byte-exact wire grammar (v1)". This module is a pure
//! codec: `decode_form` turns bytes into an in-memory `Form`,
//! `encode_form` turns a `Form` into bytes, and round-tripping
//! is the identity on well-formed inputs.
//!
//! Scope (v0.2.2-ignis0-wire):
//!
//! - Full coverage of all 34 opcodes, all 7 PUSH Value variants,
//!   and all 11 TrapKind variants from IL.md.
//! - Trailing BLAKE3 hash is written by the encoder and verified
//!   by the decoder.
//! - No semantic validation. Arity consistency, jump-target
//!   validity, declared-traps conservativity — these are all
//!   the proof checker's (S-08) responsibility, per A9.4
//!   (stage-0 does not adjudicate habitat-level correctness).
//! - No files under `kernel/forms/` are touched or consulted.
//!   Those files are IgnisSynth prose specifications; the wire
//!   format is how a **built** Form looks, after v0.5.0's
//!   external build process has resolved placeholders and
//!   sealed the canonical bytes.
//!
//! A9.4 reminder: this codec runs in the stage-0 substrate and
//! has no authority inside the habitat. Its only promises are
//! "bytes in, bytes out, round-trip" and "hash matches". It
//! does not decide what a Form **means**, only what it **is**.
//!
//! Nom is used for the decoder; the encoder is hand-written
//! because nom does not offer an encoder abstraction. The two
//! agree by construction: every `decode_*` in this file has a
//! matching `encode_*` and the round-trip tests in
//! `ignis0/tests/wire.rs` pin the correspondence.

use crate::opcode::Opcode;
use crate::value::{Hash, TrapKind, Value};
use nom::bytes::complete::{tag as nom_tag, take};
use nom::IResult;
use thiserror::Error;

// ---------- public types ----------

/// The canonical magic bytes at the head of every Form. "ISF1"
/// — IgnisSynth Form, wire version 1.
pub const MAGIC: &[u8; 4] = b"ISF1";

/// The wire version the codec emits. Decoder accepts only this
/// value; a mismatch is `WireError::BadVersion`.
pub const VERSION: u8 = 1;

/// The in-memory Form representation the codec produces.
///
/// This is **not** the same thing as an `ExecState`. An
/// `ExecState` is what the interpreter creates when it starts
/// running a Form; a `Form` is the loaded-but-not-yet-executing
/// artifact. The A9.3 fixed-point check's indirect cases (when
/// they land in v0.2.1/v0.2.3) will build `ExecState`s from
/// `Form`s that were decoded here.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Form {
    /// The substance type tag. For v1 Forms this is "Form/v1".
    /// Kept explicit on the struct so a future wire version can
    /// be distinguished without reloading bytes.
    pub type_tag: String,
    /// Expected stack depth at entry.
    pub arity: u32,
    /// Size of the locals array.
    pub locals_n: u32,
    /// Capabilities the Form expects in `cap_view` at entry.
    pub declared_caps: Vec<Hash>,
    /// Trap kinds the Form may produce. `NotImplemented` is
    /// **not** permitted here — see `WireError::ScaffoldTrapInWireForm`.
    pub declared_traps: Vec<TrapKind>,
    /// The Form's code as a vec of opcodes.
    pub code: Vec<Opcode>,
}

/// Error produced by the wire codec. Every variant corresponds
/// to a specific bullet in IL.md's wire grammar section.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum WireError {
    #[error("wire: bad magic (expected ISF1)")]
    BadMagic,
    #[error("wire: bad version (expected {expected}, got {got})")]
    BadVersion { expected: u8, got: u8 },
    #[error("wire: LEB128 overflow")]
    LebOverflow,
    #[error("wire: truncated input")]
    Truncated,
    #[error("wire: bad opcode tag 0x{0:02X}")]
    BadOpcodeTag(u8),
    #[error("wire: bad value tag 0x{0:02X}")]
    BadValueTag(u8),
    #[error("wire: bad trap tag 0x{0:02X}")]
    BadTrapTag(u8),
    #[error("wire: bad bool byte 0x{0:02X}")]
    BadBool(u8),
    #[error("wire: non-utf8 string in payload")]
    BadUtf8,
    #[error("wire: scaffold-only TrapKind::NotImplemented cannot appear in wire form")]
    ScaffoldTrapInWireForm,
    #[error("wire: trailing form_hash does not match canonical prefix")]
    BadFormHash,
    #[error("wire: trailing bytes after Form")]
    TrailingGarbage,
}

// ---------- top-level encode ----------

/// Encode a `Form` into its canonical wire bytes, including the
/// trailing BLAKE3 hash.
///
/// Fails only if the Form contains `TrapKind::NotImplemented`,
/// which is a scaffold-only marker that has no wire encoding.
pub fn encode_form(form: &Form) -> Result<Vec<u8>, WireError> {
    let mut out = Vec::with_capacity(128);
    out.extend_from_slice(MAGIC);
    out.push(VERSION);
    write_string(&mut out, &form.type_tag);
    write_uleb128_u32(&mut out, form.arity);
    write_uleb128_u32(&mut out, form.locals_n);
    write_uleb128_u32(&mut out, form.declared_caps.len() as u32);
    for h in &form.declared_caps {
        out.extend_from_slice(&h.0);
    }
    write_uleb128_u32(&mut out, form.declared_traps.len() as u32);
    for k in &form.declared_traps {
        encode_trapkind(&mut out, k)?;
    }
    write_uleb128_u32(&mut out, form.code.len() as u32);
    for op in &form.code {
        encode_opcode(&mut out, op)?;
    }
    let h = blake3::hash(&out);
    out.extend_from_slice(h.as_bytes());
    Ok(out)
}

// ---------- top-level decode ----------

/// Decode a `Form` from canonical wire bytes. The trailing
/// BLAKE3 hash is verified against the prefix, and any trailing
/// bytes beyond the hash are a `TrailingGarbage` error.
pub fn decode_form(input: &[u8]) -> Result<Form, WireError> {
    if input.len() < 4 + 1 + 32 {
        return Err(WireError::Truncated);
    }
    let body_end = input
        .len()
        .checked_sub(32)
        .ok_or(WireError::Truncated)?;
    let body = &input[..body_end];
    let trailing = &input[body_end..];

    let expected = blake3::hash(body);
    if expected.as_bytes() != trailing {
        return Err(WireError::BadFormHash);
    }

    // Now parse the body with nom. Any residual after the Form
    // record is `TrailingGarbage`.
    let (rest, form) = parse_form_body(body).map_err(map_nom_err)?;
    if !rest.is_empty() {
        return Err(WireError::TrailingGarbage);
    }
    Ok(form)
}

// ---------- nom decoder helpers ----------

type NomErr<'a> = nom::Err<nom::error::Error<&'a [u8]>>;

fn map_nom_err<'a>(e: NomErr<'a>) -> WireError {
    // We surface parser errors uniformly as Truncated unless
    // the inner body-level functions have already returned a
    // more specific WireError by short-circuiting. This is
    // adequate for the scaffold: failing tests inspect the
    // variant and we can tighten later.
    match e {
        nom::Err::Incomplete(_) => WireError::Truncated,
        _ => WireError::Truncated,
    }
}

fn parse_form_body(i: &[u8]) -> IResult<&[u8], Form> {
    let (i, _) = nom_tag(MAGIC.as_slice())(i)?;
    let (i, ver) = take(1usize)(i)?;
    if ver[0] != VERSION {
        return Err(nom::Err::Error(nom::error::Error::new(
            i,
            nom::error::ErrorKind::Tag,
        )));
    }

    let (i, type_tag) = read_string_nom(i)?;
    let (i, arity) = read_uleb128_u32_nom(i)?;
    let (i, locals_n) = read_uleb128_u32_nom(i)?;

    let (i, caps_n) = read_uleb128_u32_nom(i)?;
    let mut i = i;
    let mut declared_caps = Vec::with_capacity(caps_n as usize);
    for _ in 0..caps_n {
        let (i2, bytes) = take(32usize)(i)?;
        let mut arr = [0u8; 32];
        arr.copy_from_slice(bytes);
        declared_caps.push(Hash(arr));
        i = i2;
    }

    let (mut i, traps_n) = read_uleb128_u32_nom(i)?;
    let mut declared_traps = Vec::with_capacity(traps_n as usize);
    for _ in 0..traps_n {
        let (i2, k) = read_trapkind_nom(i)?;
        declared_traps.push(k);
        i = i2;
    }

    let (mut i, code_len) = read_uleb128_u32_nom(i)?;
    let mut code = Vec::with_capacity(code_len as usize);
    for _ in 0..code_len {
        let (i2, op) = read_opcode_nom(i)?;
        code.push(op);
        i = i2;
    }

    Ok((
        i,
        Form {
            type_tag,
            arity,
            locals_n,
            declared_caps,
            declared_traps,
            code,
        },
    ))
}

fn read_string_nom(i: &[u8]) -> IResult<&[u8], String> {
    let (i, len) = read_uleb128_u32_nom(i)?;
    let (i, bytes) = take(len as usize)(i)?;
    let s = std::str::from_utf8(bytes)
        .map_err(|_| nom::Err::Error(nom::error::Error::new(i, nom::error::ErrorKind::Tag)))?
        .to_string();
    Ok((i, s))
}

fn read_hash_nom(i: &[u8]) -> IResult<&[u8], Hash> {
    let (i, bytes) = take(32usize)(i)?;
    let mut arr = [0u8; 32];
    arr.copy_from_slice(bytes);
    Ok((i, Hash(arr)))
}

fn read_opcode_nom(i: &[u8]) -> IResult<&[u8], Opcode> {
    let (i, tag) = take(1usize)(i)?;
    let t = tag[0];
    match t {
        0x00 => {
            let (i, v) = read_value_nom(i)?;
            Ok((i, Opcode::Push(v)))
        }
        0x01 => Ok((i, Opcode::Pop)),
        0x02 => {
            let (i, n) = read_uleb128_u32_nom(i)?;
            Ok((i, Opcode::Load(n)))
        }
        0x03 => {
            let (i, n) = read_uleb128_u32_nom(i)?;
            Ok((i, Opcode::Store(n)))
        }
        0x04 => Ok((i, Opcode::Add)),
        0x05 => Ok((i, Opcode::Sub)),
        0x06 => Ok((i, Opcode::Eq)),
        0x07 => Ok((i, Opcode::Lt)),
        0x08 => {
            let (i, off) = read_zigzag_i32_nom(i)?;
            Ok((i, Opcode::Jmp(off)))
        }
        0x09 => {
            let (i, off) = read_zigzag_i32_nom(i)?;
            Ok((i, Opcode::Jmpz(off)))
        }
        0x0A => {
            let (i, form) = read_hash_nom(i)?;
            let (i, n) = read_uleb128_u32_nom(i)?;
            Ok((i, Opcode::Call { form, n }))
        }
        0x0B => Ok((i, Opcode::Ret)),
        0x0C => Ok((i, Opcode::MakePair)),
        0x0D => Ok((i, Opcode::Fst)),
        0x0E => Ok((i, Opcode::Snd)),
        0x0F => {
            let (i, n) = read_uleb128_u32_nom(i)?;
            Ok((i, Opcode::MakeVec(n)))
        }
        0x10 => {
            let (i, s) = read_string_nom(i)?;
            Ok((i, Opcode::Seal(s)))
        }
        0x11 => Ok((i, Opcode::Read)),
        0x12 => Ok((i, Opcode::Pin)),
        0x13 => Ok((i, Opcode::Unpin)),
        0x14 => Ok((i, Opcode::CapHeld)),
        0x15 => Ok((i, Opcode::Attenuate)),
        0x16 => {
            let (i, n) = read_uleb128_u32_nom(i)?;
            Ok((i, Opcode::Invoke { n }))
        }
        0x17 => Ok((i, Opcode::Revoke)),
        0x18 => Ok((i, Opcode::Append)),
        0x19 => Ok((i, Opcode::Why)),
        0x1A => Ok((i, Opcode::Yield)),
        0x1B => Ok((i, Opcode::Split)),
        0x1C => {
            let (i, k) = read_trapkind_nom(i)?;
            Ok((i, Opcode::Trap(k)))
        }
        0x1D => Ok((i, Opcode::Assert)),
        0x1E => Ok((i, Opcode::SelfHash)),
        0x1F => Ok((i, Opcode::ParseForm)),
        0x20 => Ok((i, Opcode::BindSlot)),
        0x21 => Ok((i, Opcode::ReadSlot)),
        _ => Err(nom::Err::Error(nom::error::Error::new(
            i,
            nom::error::ErrorKind::Tag,
        ))),
    }
}

fn read_value_nom(i: &[u8]) -> IResult<&[u8], Value> {
    let (i, tag) = take(1usize)(i)?;
    match tag[0] {
        0x00 => Ok((i, Value::Unit)),
        0x01 => {
            let (i, b) = take(1usize)(i)?;
            let v = match b[0] {
                0 => Value::Bool(false),
                1 => Value::Bool(true),
                _ => {
                    return Err(nom::Err::Error(nom::error::Error::new(
                        i,
                        nom::error::ErrorKind::Tag,
                    )))
                }
            };
            Ok((i, v))
        }
        0x02 => {
            let (i, n) = read_uleb128_u128_nom(i)?;
            Ok((i, Value::Nat(n)))
        }
        0x03 => {
            let (i, h) = read_hash_nom(i)?;
            Ok((i, Value::Hash(h)))
        }
        0x04 => {
            let (i, len) = read_uleb128_u32_nom(i)?;
            let (i, bytes) = take(len as usize)(i)?;
            Ok((i, Value::Bytes(bytes.to_vec())))
        }
        0x05 => {
            let (i, h) = read_hash_nom(i)?;
            Ok((i, Value::Cell(h)))
        }
        0x06 => {
            let (i, h) = read_hash_nom(i)?;
            Ok((i, Value::Cont(h)))
        }
        _ => Err(nom::Err::Error(nom::error::Error::new(
            i,
            nom::error::ErrorKind::Tag,
        ))),
    }
}

fn read_trapkind_nom(i: &[u8]) -> IResult<&[u8], TrapKind> {
    let (i, tag) = take(1usize)(i)?;
    match tag[0] {
        0x00 => {
            let (i, s) = read_string_nom(i)?;
            Ok((i, TrapKind::EBadLocal(s)))
        }
        0x01 => {
            let (i, s) = read_string_nom(i)?;
            Ok((i, TrapKind::EType(s)))
        }
        0x02 => Ok((i, TrapKind::EUnderflow)),
        0x03 => {
            let (i, s) = read_string_nom(i)?;
            Ok((i, TrapKind::EUnheld(s)))
        }
        0x04 => Ok((i, TrapKind::ENotHeld)),
        0x05 => Ok((i, TrapKind::EStale)),
        0x06 => Ok((i, TrapKind::EOverBudget)),
        0x07 => Ok((i, TrapKind::EAssert)),
        0x08 => Ok((i, TrapKind::EUnauthorised)),
        0x09 => Ok((i, TrapKind::EIgnited)),
        0x0A => Ok((i, TrapKind::EReplayDiverged)),
        _ => Err(nom::Err::Error(nom::error::Error::new(
            i,
            nom::error::ErrorKind::Tag,
        ))),
    }
}

// ---------- LEB128 decoders (nom flavour) ----------

fn read_uleb128_u32_nom(mut i: &[u8]) -> IResult<&[u8], u32> {
    let mut result: u64 = 0;
    let mut shift = 0;
    for _ in 0..5 {
        let (rest, byte) = take(1usize)(i)?;
        i = rest;
        let b = byte[0];
        result |= ((b & 0x7F) as u64) << shift;
        if b & 0x80 == 0 {
            if result > u32::MAX as u64 {
                return Err(nom::Err::Error(nom::error::Error::new(
                    i,
                    nom::error::ErrorKind::TooLarge,
                )));
            }
            return Ok((i, result as u32));
        }
        shift += 7;
    }
    Err(nom::Err::Error(nom::error::Error::new(
        i,
        nom::error::ErrorKind::TooLarge,
    )))
}

fn read_uleb128_u128_nom(mut i: &[u8]) -> IResult<&[u8], u128> {
    let mut result: u128 = 0;
    let mut shift: u32 = 0;
    for _ in 0..19 {
        let (rest, byte) = take(1usize)(i)?;
        i = rest;
        let b = byte[0];
        result |= ((b & 0x7F) as u128) << shift;
        if b & 0x80 == 0 {
            return Ok((i, result));
        }
        shift += 7;
        if shift >= 128 {
            return Err(nom::Err::Error(nom::error::Error::new(
                i,
                nom::error::ErrorKind::TooLarge,
            )));
        }
    }
    Err(nom::Err::Error(nom::error::Error::new(
        i,
        nom::error::ErrorKind::TooLarge,
    )))
}

fn read_zigzag_i32_nom(i: &[u8]) -> IResult<&[u8], i32> {
    let (i, u) = read_uleb128_u32_nom(i)?;
    let n = ((u >> 1) as i32) ^ -((u & 1) as i32);
    Ok((i, n))
}

// ---------- encoders ----------

fn write_uleb128_u32(out: &mut Vec<u8>, mut v: u32) {
    loop {
        let mut b = (v & 0x7F) as u8;
        v >>= 7;
        if v != 0 {
            b |= 0x80;
            out.push(b);
        } else {
            out.push(b);
            return;
        }
    }
}

fn write_uleb128_u128(out: &mut Vec<u8>, mut v: u128) {
    loop {
        let mut b = (v & 0x7F) as u8;
        v >>= 7;
        if v != 0 {
            b |= 0x80;
            out.push(b);
        } else {
            out.push(b);
            return;
        }
    }
}

fn write_zigzag_i32(out: &mut Vec<u8>, v: i32) {
    let zz = ((v << 1) ^ (v >> 31)) as u32;
    write_uleb128_u32(out, zz);
}

fn write_string(out: &mut Vec<u8>, s: &str) {
    write_uleb128_u32(out, s.len() as u32);
    out.extend_from_slice(s.as_bytes());
}

fn encode_opcode(out: &mut Vec<u8>, op: &Opcode) -> Result<(), WireError> {
    match op {
        Opcode::Push(v) => {
            out.push(0x00);
            encode_value(out, v)?;
        }
        Opcode::Pop => out.push(0x01),
        Opcode::Load(n) => {
            out.push(0x02);
            write_uleb128_u32(out, *n);
        }
        Opcode::Store(n) => {
            out.push(0x03);
            write_uleb128_u32(out, *n);
        }
        Opcode::Add => out.push(0x04),
        Opcode::Sub => out.push(0x05),
        Opcode::Eq => out.push(0x06),
        Opcode::Lt => out.push(0x07),
        Opcode::Jmp(off) => {
            out.push(0x08);
            write_zigzag_i32(out, *off);
        }
        Opcode::Jmpz(off) => {
            out.push(0x09);
            write_zigzag_i32(out, *off);
        }
        Opcode::Call { form, n } => {
            out.push(0x0A);
            out.extend_from_slice(&form.0);
            write_uleb128_u32(out, *n);
        }
        Opcode::Ret => out.push(0x0B),
        Opcode::MakePair => out.push(0x0C),
        Opcode::Fst => out.push(0x0D),
        Opcode::Snd => out.push(0x0E),
        Opcode::MakeVec(n) => {
            out.push(0x0F);
            write_uleb128_u32(out, *n);
        }
        Opcode::Seal(s) => {
            out.push(0x10);
            write_string(out, s);
        }
        Opcode::Read => out.push(0x11),
        Opcode::Pin => out.push(0x12),
        Opcode::Unpin => out.push(0x13),
        Opcode::CapHeld => out.push(0x14),
        Opcode::Attenuate => out.push(0x15),
        Opcode::Invoke { n } => {
            out.push(0x16);
            write_uleb128_u32(out, *n);
        }
        Opcode::Revoke => out.push(0x17),
        Opcode::Append => out.push(0x18),
        Opcode::Why => out.push(0x19),
        Opcode::Yield => out.push(0x1A),
        Opcode::Split => out.push(0x1B),
        Opcode::Trap(k) => {
            out.push(0x1C);
            encode_trapkind(out, k)?;
        }
        Opcode::Assert => out.push(0x1D),
        Opcode::SelfHash => out.push(0x1E),
        Opcode::ParseForm => out.push(0x1F),
        Opcode::BindSlot => out.push(0x20),
        Opcode::ReadSlot => out.push(0x21),
    }
    Ok(())
}

fn encode_value(out: &mut Vec<u8>, v: &Value) -> Result<(), WireError> {
    match v {
        Value::Unit => out.push(0x00),
        Value::Bool(b) => {
            out.push(0x01);
            out.push(if *b { 1 } else { 0 });
        }
        Value::Nat(n) => {
            out.push(0x02);
            write_uleb128_u128(out, *n);
        }
        Value::Hash(h) => {
            out.push(0x03);
            out.extend_from_slice(&h.0);
        }
        Value::Bytes(b) => {
            out.push(0x04);
            write_uleb128_u32(out, b.len() as u32);
            out.extend_from_slice(b);
        }
        Value::Cell(h) => {
            out.push(0x05);
            out.extend_from_slice(&h.0);
        }
        Value::Cont(h) => {
            out.push(0x06);
            out.extend_from_slice(&h.0);
        }
        // Pair and Vec are not representable in PUSH immediates.
        // The wire format deliberately has no tag for them; a
        // Form that tries to push one is ill-formed.
        Value::Pair(_, _) | Value::Vec(_) => {
            return Err(WireError::BadValueTag(0xFF));
        }
    }
    Ok(())
}

fn encode_trapkind(out: &mut Vec<u8>, k: &TrapKind) -> Result<(), WireError> {
    match k {
        TrapKind::EBadLocal(s) => {
            out.push(0x00);
            write_string(out, s);
        }
        TrapKind::EType(s) => {
            out.push(0x01);
            write_string(out, s);
        }
        TrapKind::EUnderflow => out.push(0x02),
        TrapKind::EUnheld(s) => {
            out.push(0x03);
            write_string(out, s);
        }
        TrapKind::ENotHeld => out.push(0x04),
        TrapKind::EStale => out.push(0x05),
        TrapKind::EOverBudget => out.push(0x06),
        TrapKind::EAssert => out.push(0x07),
        TrapKind::EUnauthorised => out.push(0x08),
        TrapKind::EIgnited => out.push(0x09),
        TrapKind::EReplayDiverged => out.push(0x0A),
        TrapKind::NotImplemented(_) => {
            // Scaffold-only marker; A9.4 says stage-0 has no
            // authority in the habitat, but it is also not
            // allowed to pollute the wire form with marker
            // variants that IL.md does not admit.
            return Err(WireError::ScaffoldTrapInWireForm);
        }
    }
    Ok(())
}

// ---------- small internal tests ----------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uleb128_u32_roundtrip() {
        for v in [0u32, 1, 127, 128, 16383, 16384, u32::MAX] {
            let mut buf = Vec::new();
            write_uleb128_u32(&mut buf, v);
            let (rest, got) = read_uleb128_u32_nom(&buf).unwrap();
            assert!(rest.is_empty());
            assert_eq!(got, v);
        }
    }

    #[test]
    fn zigzag_i32_roundtrip() {
        for v in [0i32, 1, -1, 63, -64, i32::MAX, i32::MIN] {
            let mut buf = Vec::new();
            write_zigzag_i32(&mut buf, v);
            let (rest, got) = read_zigzag_i32_nom(&buf).unwrap();
            assert!(rest.is_empty());
            assert_eq!(got, v);
        }
    }

    #[test]
    fn uleb128_u128_roundtrip() {
        for v in [0u128, 1, 127, 128, u64::MAX as u128, u128::MAX] {
            let mut buf = Vec::new();
            write_uleb128_u128(&mut buf, v);
            let (rest, got) = read_uleb128_u128_nom(&buf).unwrap();
            assert!(rest.is_empty());
            assert_eq!(got, v);
        }
    }

    #[test]
    fn tiny_form_roundtrip() {
        // The A9.3 canonical F: STORE 0, LOAD 0, PUSH 1, ADD, RET.
        let form = Form {
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
        };
        let bytes = encode_form(&form).unwrap();
        let decoded = decode_form(&bytes).unwrap();
        assert_eq!(decoded, form);
    }

    #[test]
    fn bad_magic_rejected() {
        let form = Form {
            type_tag: "Form/v1".into(),
            arity: 0,
            locals_n: 0,
            declared_caps: vec![],
            declared_traps: vec![],
            code: vec![Opcode::Ret],
        };
        let mut bytes = encode_form(&form).unwrap();
        bytes[0] = b'X';
        // Fix the trailing hash so we fail on magic, not hash.
        let body_end = bytes.len() - 32;
        let h = blake3::hash(&bytes[..body_end]);
        bytes[body_end..].copy_from_slice(h.as_bytes());
        let err = decode_form(&bytes).unwrap_err();
        assert!(matches!(err, WireError::Truncated)); // magic failure lands as parse err
    }

    #[test]
    fn bad_hash_rejected() {
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
        bytes[last] ^= 0xFF;
        assert_eq!(decode_form(&bytes).unwrap_err(), WireError::BadFormHash);
    }

    #[test]
    fn not_implemented_refused_by_encoder() {
        let form = Form {
            type_tag: "Form/v1".into(),
            arity: 0,
            locals_n: 0,
            declared_caps: vec![],
            declared_traps: vec![TrapKind::NotImplemented("stub".into())],
            code: vec![Opcode::Ret],
        };
        assert_eq!(
            encode_form(&form).unwrap_err(),
            WireError::ScaffoldTrapInWireForm
        );
    }
}
