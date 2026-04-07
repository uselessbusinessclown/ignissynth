//! Line-oriented Form parser — scaffold.
//!
//! This is NOT the canonical wire-form parser specified by
//! `../../kernel/IL.md` § Encoding and implemented at the Form
//! level in `../../kernel/forms/helpers/parser.form`. That one
//! reads the full `(form :name ... :code (...))` s-expression
//! and produces a sealed `ParsedForm/v1` substance.
//!
//! This scaffold reads a simpler line-oriented grammar: one
//! opcode per line, whitespace-separated tokens, `;`-prefixed
//! comments ignored. It is enough to drive the scaffold's own
//! tests and to hand-write small test Forms from within Rust
//! code or a `.il` text file.
//!
//! Grammar:
//!
//!     form     = line*
//!     line     = comment | blank | instruction
//!     comment  = ';' ...
//!     blank    = whitespace only
//!     instruction = mnemonic (whitespace operand)*
//!
//! Examples:
//!
//!     ; add 1 to input
//!     STORE 0
//!     LOAD 0
//!     PUSH 1
//!     ADD
//!     RET
//!
//! When the canonical parser lands (as a proper IL-encoded
//! helper chain under ignis0), this file will be deleted.

use crate::opcode::Opcode;
use crate::value::Value;

/// Parse a line-oriented Form source into a vec of opcodes.
///
/// Returns `Err` on the first unparseable line, with the line
/// number (1-indexed) and the offending text.
pub fn parse_form_lines(source: &str) -> Result<Vec<Opcode>, ParseError> {
    let mut ops = Vec::new();
    for (i, raw_line) in source.lines().enumerate() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with(';') {
            continue;
        }
        match parse_line(line) {
            Some(op) => ops.push(op),
            None => {
                return Err(ParseError {
                    line_number: i + 1,
                    line: raw_line.to_string(),
                })
            }
        }
    }
    Ok(ops)
}

/// One line's parse error.
#[derive(Debug, Clone, thiserror::Error)]
#[error("parse error at line {line_number}: {line:?}")]
pub struct ParseError {
    pub line_number: usize,
    pub line: String,
}

fn parse_line(line: &str) -> Option<Opcode> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.is_empty() {
        return None;
    }
    match parts[0] {
        // Stack and locals
        "PUSH" => parts.get(1).and_then(|s| s.parse::<u128>().ok()).map(|n| Opcode::Push(Value::Nat(n))),
        "POP" => Some(Opcode::Pop),
        "LOAD" => parts.get(1).and_then(|s| s.parse::<u32>().ok()).map(Opcode::Load),
        "STORE" => parts.get(1).and_then(|s| s.parse::<u32>().ok()).map(Opcode::Store),

        // Arithmetic
        "ADD" => Some(Opcode::Add),
        "SUB" => Some(Opcode::Sub),
        "EQ" => Some(Opcode::Eq),
        "LT" => Some(Opcode::Lt),

        // Control flow
        "JMP" => parts.get(1).and_then(|s| s.parse::<i32>().ok()).map(Opcode::Jmp),
        "JMPZ" => parts.get(1).and_then(|s| s.parse::<i32>().ok()).map(Opcode::Jmpz),
        "RET" => Some(Opcode::Ret),
        // CALL intentionally omitted — needs hash parsing which
        // the line-oriented scaffold does not support.

        // Structure
        "MAKEPAIR" => Some(Opcode::MakePair),
        "FST" => Some(Opcode::Fst),
        "SND" => Some(Opcode::Snd),
        "MAKEVEC" => parts.get(1).and_then(|s| s.parse::<u32>().ok()).map(Opcode::MakeVec),

        // Substance
        "SEAL" => parts.get(1).map(|t| Opcode::Seal(t.to_string())),
        "READ" => Some(Opcode::Read),
        "PIN" => Some(Opcode::Pin),
        "UNPIN" => Some(Opcode::Unpin),

        // Attention
        "YIELD" => Some(Opcode::Yield),

        // Trap/Assert
        "TRAP" => {
            // Default to a NotImplemented trap with the named kind
            // as context. The scaffold doesn't have a trap-kind
            // enumeration parser.
            let kind = parts.get(1).unwrap_or(&"unknown");
            Some(Opcode::Trap(crate::value::TrapKind::NotImplemented(
                kind.to_string(),
            )))
        }
        "ASSERT" => Some(Opcode::Assert),

        // Reflection
        "SELFHASH" => Some(Opcode::SelfHash),

        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_canonical_f() {
        let source = r#"
            ; canonical F from IGNITION-BOOTSTRAP.md § Step 2
            STORE 0
            LOAD 0
            PUSH 1
            ADD
            RET
        "#;
        let ops = parse_form_lines(source).unwrap();
        assert_eq!(ops.len(), 5);
        assert!(matches!(ops[0], Opcode::Store(0)));
        assert!(matches!(ops[1], Opcode::Load(0)));
        assert!(matches!(ops[2], Opcode::Push(Value::Nat(1))));
        assert!(matches!(ops[3], Opcode::Add));
        assert!(matches!(ops[4], Opcode::Ret));
    }

    #[test]
    fn rejects_unknown_mnemonic() {
        let source = "FOOBAR 42";
        assert!(parse_form_lines(source).is_err());
    }

    #[test]
    fn ignores_comments_and_blanks() {
        let source = r#"
            ; comment
            ;
            PUSH 7
            RET
        "#;
        let ops = parse_form_lines(source).unwrap();
        assert_eq!(ops.len(), 2);
    }
}
