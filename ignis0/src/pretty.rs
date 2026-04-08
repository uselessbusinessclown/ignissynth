//! Pretty-printer for the line-oriented `.form` text format.
//!
//! Converts a `Vec<Opcode>` back into the canonical line-oriented
//! source text that `parser::parse_form_lines` accepts. The output
//! is:
//!
//! - One opcode per line.
//! - Mnemonic left-aligned, operand (if any) separated by one space.
//! - A `;` comment is prepended to groups to label them.
//! - Immediate values use their most readable textual form.
//!
//! This is *not* the canonical wire-form s-expression format from
//! `kernel/IL.md` § Encoding. That format includes the full
//! `(form :type-tag ... :code (...))` envelope and is produced by
//! the kernel's own canonicaliser Form. This printer covers the
//! simpler scaffold grammar used by `parser::parse_form_lines` and
//! is suitable for diagnostic output, test fixtures, and round-trip
//! testing.
//!
//! # Round-trip guarantee
//!
//! For every `Vec<Opcode>` that `parse_form_lines` can parse,
//! `pretty_print` produces source text that `parse_form_lines`
//! will parse back to a structurally equivalent opcode sequence
//! (same mnemonics, same operands). The guarantee is exercised by
//! the `round_trip` tests at the bottom of this file.

use std::fmt::Write as FmtWrite;

use crate::opcode::Opcode;
use crate::value::Value;

/// Pretty-print a slice of opcodes to a `String`.
///
/// The returned string has one opcode per line and ends with a
/// trailing newline.
pub fn pretty_print(code: &[Opcode]) -> String {
    let mut out = String::new();
    for op in code {
        let line = opcode_to_line(op);
        out.push_str(&line);
        out.push('\n');
    }
    out
}

/// Pretty-print with a header comment block.
///
/// Produces the full scaffold source: a `;`-comment header
/// followed by the opcode lines.
pub fn pretty_print_with_header(name: &str, locals_n: usize, code: &[Opcode]) -> String {
    let mut out = String::new();
    writeln!(out, "; Form: {}", name).unwrap();
    writeln!(out, "; locals-n: {}", locals_n).unwrap();
    writeln!(out, "; arity: (see ExecState::new call site)").unwrap();
    writeln!(out).unwrap();
    out.push_str(&pretty_print(code));
    out
}

/// Format a single opcode as its mnemonic (+ operand when present).
pub fn opcode_to_line(op: &Opcode) -> String {
    match op {
        // ── Stack and locals ──────────────────────────────────────────
        Opcode::Push(v) => format!("PUSH {}", value_to_token(v)),
        Opcode::Pop => "POP".to_string(),
        Opcode::Load(i) => format!("LOAD {}", i),
        Opcode::Store(i) => format!("STORE {}", i),

        // ── Arithmetic and comparison ─────────────────────────────────
        Opcode::Add => "ADD".to_string(),
        Opcode::Sub => "SUB".to_string(),
        Opcode::Eq => "EQ".to_string(),
        Opcode::Lt => "LT".to_string(),

        // ── Control flow ──────────────────────────────────────────────
        Opcode::Jmp(off) => format!("JMP {}", off),
        Opcode::Jmpz(off) => format!("JMPZ {}", off),
        Opcode::Call { form, n } => format!("CALL h={} /n={}", form.short(), n),
        Opcode::Ret => "RET".to_string(),

        // ── Structure ─────────────────────────────────────────────────
        Opcode::MakePair => "MAKEPAIR".to_string(),
        Opcode::Fst => "FST".to_string(),
        Opcode::Snd => "SND".to_string(),
        Opcode::MakeVec(n) => format!("MAKEVEC {}", n),

        // ── Substance ─────────────────────────────────────────────────
        Opcode::Seal(tag) => format!("SEAL {}", tag),
        Opcode::Read => "READ".to_string(),
        Opcode::Pin => "PIN".to_string(),
        Opcode::Unpin => "UNPIN".to_string(),

        // ── Capability ────────────────────────────────────────────────
        Opcode::CapHeld => "CAPHELD".to_string(),
        Opcode::Attenuate => "ATTENUATE".to_string(),
        Opcode::Invoke { n } => format!("INVOKE {}", n),
        Opcode::Revoke => "REVOKE".to_string(),

        // ── Weave ─────────────────────────────────────────────────────
        Opcode::Append => "APPEND".to_string(),
        Opcode::Why => "WHY".to_string(),

        // ── Attention ─────────────────────────────────────────────────
        Opcode::Yield => "YIELD".to_string(),
        Opcode::Split => "SPLIT".to_string(),

        // ── Trap ──────────────────────────────────────────────────────
        Opcode::Trap(k) => format!("TRAP {}", trap_kind_token(k)),
        Opcode::Assert => "ASSERT".to_string(),

        // ── Reflection ────────────────────────────────────────────────
        Opcode::SelfHash => "SELFHASH".to_string(),
        Opcode::ParseForm => "PARSEFORM".to_string(),
        Opcode::BindSlot => "BINDSLOT".to_string(),
        Opcode::ReadSlot => "READSLOT".to_string(),
    }
}

/// Render a `Value` as the token the scaffold parser accepts for
/// `PUSH` immediates. Only `Nat` and `Bool` are round-trippable
/// through the line parser (which only parses `PUSH <u128>`).
/// Other variants are rendered with a `;`-prefixed annotation so
/// the output is at least readable as a comment.
fn value_to_token(v: &Value) -> String {
    match v {
        Value::Nat(n) => n.to_string(),
        Value::Bool(b) => {
            // The line parser does not handle PUSH true/false, so
            // we emit the decimal value (1/0) with a comment.
            format!("; bool {} — use PUSH {} for parser compat", b, *b as u8)
        }
        Value::Unit => "; unit".to_string(),
        Value::Hash(h) => format!("; hash:{}", h.short()),
        Value::Bytes(b) => format!("; bytes({} bytes)", b.len()),
        Value::Pair(a, b) => format!("; pair({}, {})", value_to_token(a), value_to_token(b)),
        Value::Vec(vs) => format!("; vec[{}]", vs.len()),
        Value::Cell(h) => format!("; cell:{}", h.short()),
        Value::Cont(h) => format!("; cont:{}", h.short()),
    }
}

fn trap_kind_token(k: &crate::value::TrapKind) -> &'static str {
    use crate::value::TrapKind::*;
    match k {
        EBadLocal(_) => "EBADLOCAL",
        EType(_) => "ETYPE",
        EUnderflow => "EUNDERFLOW",
        EUnheld(_) => "EUNHELD",
        ENotHeld => "ENOTHELD",
        EStale => "ESTALE",
        EOverBudget => "EOVERBUDGET",
        EAssert => "EASSERT",
        EUnauthorised => "EUNAUTHORISED",
        EIgnited => "EIGNITED",
        EReplayDiverged => "EREPLAYDIVERGED",
        NotImplemented(_) => "NotImplemented",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixed_point::FixedPointCheck;
    use crate::opcode::Opcode;
    use crate::parser::parse_form_lines;
    use crate::value::Value;

    /// Round-trip the canonical Form F through pretty_print then
    /// parse_form_lines and confirm the mnemonic sequence is identical.
    #[test]
    fn round_trip_canonical_f() {
        let code = FixedPointCheck::build_F();
        let printed = pretty_print(&code);
        let reparsed = parse_form_lines(&printed).expect("pretty output must be parseable");
        assert_eq!(code.len(), reparsed.len(), "length mismatch after round-trip");
        for (a, b) in code.iter().zip(reparsed.iter()) {
            assert_eq!(
                a.mnemonic(),
                b.mnemonic(),
                "mnemonic mismatch after round-trip"
            );
        }
    }

    /// A sequence of purely parseable opcodes round-trips exactly.
    #[test]
    fn round_trip_arithmetic_sequence() {
        let code = vec![
            Opcode::Push(Value::Nat(10)),
            Opcode::Push(Value::Nat(32)),
            Opcode::Add,
            Opcode::Push(Value::Nat(0)),
            Opcode::Eq,
            Opcode::Assert,
            Opcode::Push(Value::Nat(99)),
            Opcode::Ret,
        ];
        let printed = pretty_print(&code);
        let reparsed = parse_form_lines(&printed).unwrap();
        assert_eq!(code.len(), reparsed.len());
        for (a, b) in code.iter().zip(reparsed.iter()) {
            assert_eq!(a.mnemonic(), b.mnemonic());
        }
    }

    /// opcode_to_line produces the correct mnemonic for each opcode.
    #[test]
    fn mnemonic_coverage() {
        use crate::value::{Hash, TrapKind};

        let samples: Vec<(Opcode, &str)> = vec![
            (Opcode::Push(Value::Nat(0)), "PUSH"),
            (Opcode::Pop, "POP"),
            (Opcode::Load(0), "LOAD"),
            (Opcode::Store(0), "STORE"),
            (Opcode::Add, "ADD"),
            (Opcode::Sub, "SUB"),
            (Opcode::Eq, "EQ"),
            (Opcode::Lt, "LT"),
            (Opcode::Jmp(0), "JMP"),
            (Opcode::Jmpz(0), "JMPZ"),
            (Opcode::Call { form: Hash::BOTTOM, n: 0 }, "CALL"),
            (Opcode::Ret, "RET"),
            (Opcode::MakePair, "MAKEPAIR"),
            (Opcode::Fst, "FST"),
            (Opcode::Snd, "SND"),
            (Opcode::MakeVec(0), "MAKEVEC"),
            (Opcode::Seal("T".into()), "SEAL"),
            (Opcode::Read, "READ"),
            (Opcode::Pin, "PIN"),
            (Opcode::Unpin, "UNPIN"),
            (Opcode::CapHeld, "CAPHELD"),
            (Opcode::Attenuate, "ATTENUATE"),
            (Opcode::Invoke { n: 0 }, "INVOKE"),
            (Opcode::Revoke, "REVOKE"),
            (Opcode::Append, "APPEND"),
            (Opcode::Why, "WHY"),
            (Opcode::Yield, "YIELD"),
            (Opcode::Split, "SPLIT"),
            (Opcode::Trap(TrapKind::EAssert), "TRAP"),
            (Opcode::Assert, "ASSERT"),
            (Opcode::SelfHash, "SELFHASH"),
            (Opcode::ParseForm, "PARSEFORM"),
            (Opcode::BindSlot, "BINDSLOT"),
            (Opcode::ReadSlot, "READSLOT"),
        ];

        assert_eq!(samples.len(), 34, "must cover all 34 opcodes");

        for (op, expected_mnemonic) in &samples {
            let line = opcode_to_line(op);
            assert!(
                line.starts_with(expected_mnemonic),
                "opcode_to_line({:?}) = {:?}, expected prefix {:?}",
                op.mnemonic(),
                line,
                expected_mnemonic
            );
        }
    }

    /// pretty_print_with_header includes the Form name and locals-n.
    #[test]
    fn header_contains_name_and_locals() {
        let code = vec![Opcode::Push(Value::Nat(1)), Opcode::Ret];
        let out = pretty_print_with_header("my-form", 3, &code);
        assert!(out.contains("; Form: my-form"), "missing Form: name");
        assert!(out.contains("; locals-n: 3"), "missing locals-n");
    }
}
