//! Integration test for the A9.3 fixed-point check.
//!
//! Exercises all three levels of the check (direct + two
//! indirection levels) using the v0.2.3-ignis0-fp machinery:
//! the canonical F from `../../kernel/IGNITION-BOOTSTRAP.md`
//! § Step 2 is registered via the wire codec and invoked
//! through hand-encoded micro-`S-07/execute` wrappers that
//! use `CALL` to reach it.

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

/// v0.2.3-ignis0-fp: full harness, all three levels live.
/// Direct, one-level indirect, and two-level indirect all
/// produce Nat(43); the frame-depth observations match the
/// expected call chain (2 for level 1, 3 for level 2).
#[test]
fn fixed_point_harness_passes_all_three_levels() {
    let mut check = FixedPointCheck::new();
    match check.run() {
        FixedPointVerdict::Pass {
            direct,
            indirect_1,
            indirect_2,
            indirect_1_max_depth,
            indirect_2_max_depth,
        } => {
            assert_eq!(direct, Value::Nat(43));
            assert_eq!(indirect_1, Value::Nat(43));
            assert_eq!(indirect_2, Value::Nat(43));
            assert_eq!(
                indirect_1_max_depth, 2,
                "indirect_1 chain should be micro_s07 → F"
            );
            assert_eq!(
                indirect_2_max_depth, 3,
                "indirect_2 chain should be micro_s07² → micro_s07 → F"
            );
        }
        other => panic!("expected Pass, got {:?}", other),
    }
}

/// Indirect level 1 in isolation: micro-S-07(F)(42) = 43.
#[test]
fn eval_indirect_1_yields_43_via_call() {
    let mut check = FixedPointCheck::new();
    match check.eval_indirect_1(42) {
        ignis0::fixed_point::EvalVerdict::ProducedTraced(v, d) => {
            assert_eq!(v, Value::Nat(43));
            assert_eq!(d, 2, "call chain: micro_s07 frame over F frame");
        }
        other => panic!("expected ProducedTraced, got {:?}", other),
    }
}

/// Indirect level 2 in isolation: micro-S-07²(F)(42) = 43
/// and the call chain reaches depth 3 (S-07² → S-07 → F).
#[test]
fn eval_indirect_2_yields_43_via_double_call() {
    let mut check = FixedPointCheck::new();
    match check.eval_indirect_2(42) {
        ignis0::fixed_point::EvalVerdict::ProducedTraced(v, d) => {
            assert_eq!(v, Value::Nat(43));
            assert_eq!(d, 3, "call chain: s07² → s07 → F");
        }
        other => panic!("expected ProducedTraced, got {:?}", other),
    }
}

/// The three paths must agree on the value, per A9.3.
#[test]
fn three_paths_agree_on_canonical_input() {
    let mut check = FixedPointCheck::new();
    let direct = check.eval_direct(42).as_nat();
    let i1 = check.eval_indirect_1(42).as_nat();
    let i2 = check.eval_indirect_2(42).as_nat();
    assert_eq!(direct, Some(43));
    assert_eq!(i1, Some(43));
    assert_eq!(i2, Some(43));
}

/// Sanity: the indirect path also works on input 0, which
/// stresses a different ADD boundary. Both levels should
/// produce Nat(1).
#[test]
fn indirect_paths_agree_on_zero_input() {
    let mut check = FixedPointCheck::new();
    assert_eq!(check.eval_indirect_1(0).as_nat(), Some(1));
    assert_eq!(check.eval_indirect_2(0).as_nat(), Some(1));
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
