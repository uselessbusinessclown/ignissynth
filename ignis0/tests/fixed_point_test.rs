//! Integration test for the A9.3 fixed-point check.
//!
//! Exercises the direct case of the canonical Form F from
//! `../../kernel/IGNITION-BOOTSTRAP.md` § Step 2. The indirect
//! cases are stubbed in this scaffold and will gain their own
//! tests once the canonical parser and CALL opcode are wired up.

use ignis0::exec::{ExecState, ExecVerdict, Interpreter};
use ignis0::fixed_point::{FixedPointCheck, FixedPointVerdict};
use ignis0::opcode::Opcode;
use ignis0::parser::parse_form_lines;
use ignis0::store::SubstanceStore;
use ignis0::value::{Hash, TrapKind, Value};

/// The canonical F from IGNITION-BOOTSTRAP.md § Step 2 must
/// produce 43 when executed directly on input 42.
#[test]
fn canonical_f_on_42_yields_43() {
    let mut store = SubstanceStore::new();
    let code = FixedPointCheck::build_F();
    let mut state = ExecState::new(Hash::BOTTOM, code, 1, vec![Value::Nat(42)]);
    let mut interp = Interpreter::new(&mut store);
    match interp.run(&mut state, 64) {
        ExecVerdict::Returned(Value::Nat(43)) => {} // expected
        other => panic!("expected Returned(Nat(43)), got {:?}", other),
    }
}

/// The same Form, but built via the scaffold parser, must
/// produce the same result on the same input.
#[test]
fn canonical_f_parsed_on_42_yields_43() {
    let mut store = SubstanceStore::new();
    let code = FixedPointCheck::build_f_parsed();
    let mut state = ExecState::new(Hash::BOTTOM, code, 1, vec![Value::Nat(42)]);
    let mut interp = Interpreter::new(&mut store);
    match interp.run(&mut state, 64) {
        ExecVerdict::Returned(Value::Nat(43)) => {}
        other => panic!("expected Returned(Nat(43)), got {:?}", other),
    }
}

/// The hand-constructed and parser-constructed versions must
/// produce the same opcode vec.
#[test]
fn parser_matches_hand_constructed() {
    let hand = FixedPointCheck::build_F();
    let parsed = FixedPointCheck::build_f_parsed();
    assert_eq!(hand.len(), parsed.len());
    // Compare mnemonic-by-mnemonic; the Opcode enum does not
    // derive PartialEq everywhere (Value::Bytes can differ),
    // but mnemonics are enough for the canonical F.
    for (a, b) in hand.iter().zip(parsed.iter()) {
        assert_eq!(a.mnemonic(), b.mnemonic());
    }
}

/// Same Form on input 0 yields 1.
#[test]
fn canonical_f_on_0_yields_1() {
    let mut store = SubstanceStore::new();
    let code = FixedPointCheck::build_F();
    let mut state = ExecState::new(Hash::BOTTOM, code, 1, vec![Value::Nat(0)]);
    let mut interp = Interpreter::new(&mut store);
    match interp.run(&mut state, 64) {
        ExecVerdict::Returned(Value::Nat(1)) => {}
        other => panic!("expected Returned(Nat(1)), got {:?}", other),
    }
}

/// F must trap `EType` if the input is not a Nat (e.g. a Bool).
/// The ADD opcode will encounter a non-Nat operand.
#[test]
fn canonical_f_on_non_nat_traps_etype() {
    let mut store = SubstanceStore::new();
    let code = FixedPointCheck::build_F();
    let mut state = ExecState::new(Hash::BOTTOM, code, 1, vec![Value::Bool(true)]);
    let mut interp = Interpreter::new(&mut store);
    match interp.run(&mut state, 64) {
        ExecVerdict::Trapped(TrapKind::EType(_)) => {}
        other => panic!("expected Trapped(EType), got {:?}", other),
    }
}

/// Full harness: direct passes, indirects stubbed.
#[test]
fn fixed_point_harness_is_incomplete_scaffold() {
    let mut check = FixedPointCheck::new();
    match check.run() {
        FixedPointVerdict::Incomplete { direct, .. } => {
            assert_eq!(direct, Value::Nat(43));
        }
        other => panic!("expected Incomplete, got {:?}", other),
    }
}

/// Store seal/seal/seal idempotency.
#[test]
fn store_seal_is_idempotent() {
    let mut store = SubstanceStore::new();
    let h1 = store.seal("TestTag", Value::Nat(7));
    let h2 = store.seal("TestTag", Value::Nat(7));
    assert_eq!(h1, h2);
    assert_eq!(store.len(), 1);
}

/// Unpin-to-zero removes the cell.
#[test]
fn store_unpin_to_zero_removes_cell() {
    let mut store = SubstanceStore::new();
    let h = store.seal("TestTag", Value::Nat(7));
    assert_eq!(store.len(), 1);
    assert_eq!(store.read(&h), Ok(Value::Nat(7)));
    store.unpin(&h).unwrap();
    assert_eq!(store.len(), 0);
    match store.read(&h) {
        Err(TrapKind::EUnheld(_)) => {}
        other => panic!("expected EUnheld, got {:?}", other),
    }
}

/// ADD on two Nats.
#[test]
fn add_works() {
    let mut store = SubstanceStore::new();
    let code = vec![
        Opcode::Push(Value::Nat(40)),
        Opcode::Push(Value::Nat(2)),
        Opcode::Add,
        Opcode::Ret,
    ];
    let mut state = ExecState::new(Hash::BOTTOM, code, 0, vec![]);
    let mut interp = Interpreter::new(&mut store);
    match interp.run(&mut state, 16) {
        ExecVerdict::Returned(Value::Nat(42)) => {}
        other => panic!("expected Returned(Nat(42)), got {:?}", other),
    }
}

/// SUB trapping on underflow is structural.
#[test]
fn sub_underflow_traps() {
    let mut store = SubstanceStore::new();
    let code = vec![
        Opcode::Push(Value::Nat(1)),
        Opcode::Push(Value::Nat(5)),
        Opcode::Sub,
        Opcode::Ret,
    ];
    let mut state = ExecState::new(Hash::BOTTOM, code, 0, vec![]);
    let mut interp = Interpreter::new(&mut store);
    match interp.run(&mut state, 16) {
        ExecVerdict::Trapped(TrapKind::EUnderflow) => {}
        other => panic!("expected Trapped(EUnderflow), got {:?}", other),
    }
}

/// The scaffold parser rejects unknown mnemonics with a
/// structured error.
#[test]
fn parser_rejects_unknown_mnemonic() {
    let result = parse_form_lines("FOOBAR 42");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.line_number, 1);
    assert_eq!(err.line, "FOOBAR 42");
}
