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
use ignis0::registry::{FormRegistry, LoadedForm};
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

/// CALL/RET round-trip: caller pushes 42, calls F (the +1
/// Form), and observes 43 on its own stack after RET.
#[test]
fn call_ret_invokes_registered_form() {
    let mut store = SubstanceStore::new();

    // Register F (+1) under a fabricated hash. The hash content
    // is irrelevant for dispatch — only equality matters here.
    let f_hash = Hash::of(b"test:F/+1");
    let mut registry = FormRegistry::new();
    registry.register(
        f_hash,
        LoadedForm {
            code: FixedPointCheck::build_F(),
            locals_n: 1,
            name: "F+1".to_string(),
        },
    );

    // Caller: push 42, CALL F with 1 arg, RET the result.
    let caller_code = vec![
        Opcode::Push(Value::Nat(42)),
        Opcode::Call { form: f_hash, n: 1 },
        Opcode::Ret,
    ];
    let mut state = ExecState::new(Hash::BOTTOM, caller_code, 0, vec![]);
    let mut interp = Interpreter::new(&mut store).with_registry(&registry);
    match interp.run(&mut state, 64) {
        ExecVerdict::Returned(Value::Nat(43)) => {}
        other => panic!("expected Returned(Nat(43)), got {:?}", other),
    }
}

/// CALL with no registry attached traps `NotImplemented` rather
/// than panicking. Documents the scaffold-only fallback.
#[test]
fn call_without_registry_traps() {
    let mut store = SubstanceStore::new();
    let code = vec![
        Opcode::Call {
            form: Hash::BOTTOM,
            n: 0,
        },
        Opcode::Ret,
    ];
    let mut state = ExecState::new(Hash::BOTTOM, code, 0, vec![]);
    let mut interp = Interpreter::new(&mut store);
    match interp.run(&mut state, 16) {
        ExecVerdict::Trapped(TrapKind::NotImplemented(_)) => {}
        other => panic!("expected Trapped(NotImplemented), got {:?}", other),
    }
}

/// CALL with an unknown form hash traps `EUnheld`.
#[test]
fn call_unknown_form_traps_eunheld() {
    let mut store = SubstanceStore::new();
    let registry = FormRegistry::new();
    let code = vec![
        Opcode::Call {
            form: Hash::of(b"nope"),
            n: 0,
        },
        Opcode::Ret,
    ];
    let mut state = ExecState::new(Hash::BOTTOM, code, 0, vec![]);
    let mut interp = Interpreter::new(&mut store).with_registry(&registry);
    match interp.run(&mut state, 16) {
        ExecVerdict::Trapped(TrapKind::EUnheld(_)) => {}
        other => panic!("expected Trapped(EUnheld), got {:?}", other),
    }
}

/// CALL with too few stack args traps `EType`.
#[test]
fn call_with_insufficient_args_traps() {
    let mut store = SubstanceStore::new();
    let f_hash = Hash::of(b"test:F/+1");
    let mut registry = FormRegistry::new();
    registry.register(
        f_hash,
        LoadedForm {
            code: FixedPointCheck::build_F(),
            locals_n: 1,
            name: "F+1".to_string(),
        },
    );
    let code = vec![
        // Want 1 arg, supply 0.
        Opcode::Call { form: f_hash, n: 1 },
        Opcode::Ret,
    ];
    let mut state = ExecState::new(Hash::BOTTOM, code, 0, vec![]);
    let mut interp = Interpreter::new(&mut store).with_registry(&registry);
    match interp.run(&mut state, 16) {
        ExecVerdict::Trapped(TrapKind::EType(_)) => {}
        other => panic!("expected Trapped(EType), got {:?}", other),
    }
}

/// Nested CALL: G calls F twice (+1 then +1) to compute +2.
#[test]
fn nested_calls_compose() {
    let mut store = SubstanceStore::new();
    let f_hash = Hash::of(b"test:F/+1");
    let mut registry = FormRegistry::new();
    registry.register(
        f_hash,
        LoadedForm {
            code: FixedPointCheck::build_F(),
            locals_n: 1,
            name: "F+1".to_string(),
        },
    );

    // G: push input, CALL F, CALL F, RET.
    let g_code = vec![
        Opcode::Store(0),
        Opcode::Load(0),
        Opcode::Call { form: f_hash, n: 1 },
        Opcode::Call { form: f_hash, n: 1 },
        Opcode::Ret,
    ];
    let mut state = ExecState::new(Hash::BOTTOM, g_code, 1, vec![Value::Nat(40)]);
    let mut interp = Interpreter::new(&mut store).with_registry(&registry);
    match interp.run(&mut state, 128) {
        ExecVerdict::Returned(Value::Nat(42)) => {}
        other => panic!("expected Returned(Nat(42)), got {:?}", other),
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

// ── Capability / INVOKE tests ─────────────────────────────────────────────

use ignis0::capability::{builtin_cap_id, CapabilityInvoker, CapabilityRegistry};

/// A minimal capability invoker for testing: doubles its single Nat arg.
struct DoubleCapability;
impl CapabilityInvoker for DoubleCapability {
    fn name(&self) -> &str {
        "test/double"
    }
    fn invoke(
        &self,
        _cap_id: Hash,
        args: Vec<Value>,
        _store: &mut SubstanceStore,
    ) -> Result<Value, ignis0::value::TrapKind> {
        match args.as_slice() {
            [Value::Nat(n)] => Ok(Value::Nat(n * 2)),
            _ => Err(ignis0::value::TrapKind::EType(
                "test/double: expected 1 Nat".into(),
            )),
        }
    }
}

/// INVOKE dispatches through the CapabilityRegistry and returns the
/// capability's result onto the caller's stack.
#[test]
fn invoke_dispatches_to_registered_capability() {
    use std::sync::Arc;

    let cap_id = builtin_cap_id(b"test/double/v1");
    let mut reg = CapabilityRegistry::new();
    reg.register(cap_id, Box::new(DoubleCapability));
    let reg = Arc::new(reg);

    let mut store = SubstanceStore::new();
    let code = vec![
        Opcode::Push(Value::Nat(21)),  // arg
        Opcode::Push(Value::Hash(cap_id)), // cap_id on top
        Opcode::Invoke { n: 1 },
        Opcode::Ret,
    ];
    let mut state = ExecState::new(Hash::BOTTOM, code, 0, vec![]);
    let mut interp = Interpreter::new(&mut store).with_cap_registry(reg);
    match interp.run(&mut state, 32) {
        ExecVerdict::Returned(Value::Nat(42)) => {}
        other => panic!("expected Returned(Nat(42)), got {:?}", other),
    }
}

/// INVOKE with no cap_registry traps ENotHeld.
#[test]
fn invoke_without_registry_traps_enotheld() {
    let mut store = SubstanceStore::new();
    let cap_id = builtin_cap_id(b"test/any");
    let code = vec![
        Opcode::Push(Value::Hash(cap_id)),
        Opcode::Invoke { n: 0 },
        Opcode::Ret,
    ];
    let mut state = ExecState::new(Hash::BOTTOM, code, 0, vec![]);
    let mut interp = Interpreter::new(&mut store);
    match interp.run(&mut state, 16) {
        ExecVerdict::Trapped(ignis0::value::TrapKind::ENotHeld) => {}
        other => panic!("expected Trapped(ENotHeld), got {:?}", other),
    }
}

/// INVOKE on an unknown cap_id (not in the registry) traps ENotHeld.
#[test]
fn invoke_unknown_cap_traps_enotheld() {
    use std::sync::Arc;

    let reg = Arc::new(CapabilityRegistry::new()); // empty
    let mut store = SubstanceStore::new();
    let cap_id = builtin_cap_id(b"test/nonexistent");
    let code = vec![
        Opcode::Push(Value::Hash(cap_id)),
        Opcode::Invoke { n: 0 },
        Opcode::Ret,
    ];
    let mut state = ExecState::new(Hash::BOTTOM, code, 0, vec![]);
    let mut interp = Interpreter::new(&mut store).with_cap_registry(reg);
    match interp.run(&mut state, 16) {
        ExecVerdict::Trapped(ignis0::value::TrapKind::ENotHeld) => {}
        other => panic!("expected Trapped(ENotHeld), got {:?}", other),
    }
}
