//! Integration tests for the v0.2.4 opcode batch:
//!   CAPHELD, ATTENUATE, REVOKE, APPEND, WHY, SPLIT,
//!   READSLOT, BINDSLOT, PARSEFORM.
//!
//! Each test exercises one opcode at the Interpreter level via a
//! minimal Form: push args, emit the opcode, RET — check the
//! ExecVerdict. Tests do not touch wire.rs, fixed_point.rs, or
//! any parser path.

use std::sync::Arc;

use ignis0::{
    capability::{CapabilityInvoker, CapabilityRegistry},
    exec::{ExecState, ExecVerdict, Interpreter},
    opcode::Opcode,
    registry::{FormRegistry, LoadedForm},
    store::SubstanceStore,
    value::{Hash, TrapKind, Value},
    wire::{encode_form, Form},
};

// ── Helpers ──────────────────────────────────────────────────────────────────

/// Minimal capability backend for tests.
struct StubCap;

impl CapabilityInvoker for StubCap {
    fn name(&self) -> &str {
        "stub"
    }
    fn invoke(
        &self,
        _cap_id: Hash,
        _args: Vec<Value>,
        _store: &mut SubstanceStore,
    ) -> Result<Value, TrapKind> {
        Ok(Value::Unit)
    }
}

/// Build a single-opcode Form + RET and run it to verdict.
fn run_single(
    op: Opcode,
    stack_inputs: Vec<Value>,
    store: &mut SubstanceStore,
) -> ExecVerdict {
    // Build: push inputs in argument order then execute the opcode.
    // ExecState::new reverses inputs so first input ends up on top —
    // that's arg0 on top, which is what single-arg opcodes expect.
    // For multi-arg opcodes we need the deeper arg pushed first.
    let code = vec![op, Opcode::Ret];
    let mut state = ExecState::new(Hash::BOTTOM, code, 0, stack_inputs);
    let mut interp = Interpreter::new(store);
    interp.run(&mut state, 1000)
}

/// Same as run_single but with a FormRegistry attached.
fn run_with_registry<'a>(
    op: Opcode,
    stack_inputs: Vec<Value>,
    store: &'a mut SubstanceStore,
    reg: &'a FormRegistry,
) -> ExecVerdict {
    let code = vec![op, Opcode::Ret];
    let mut state = ExecState::new(Hash::BOTTOM, code, 0, stack_inputs);
    let mut interp = Interpreter::new(store).with_registry(reg);
    interp.run(&mut state, 1000)
}

/// Same as run_single but with a CapabilityRegistry attached.
fn run_with_capreg(
    op: Opcode,
    stack_inputs: Vec<Value>,
    store: &mut SubstanceStore,
    cap_reg: Arc<CapabilityRegistry>,
) -> ExecVerdict {
    let code = vec![op, Opcode::Ret];
    let mut state = ExecState::new(Hash::BOTTOM, code, 0, stack_inputs);
    let mut interp = Interpreter::new(store).with_cap_registry(cap_reg);
    interp.run(&mut state, 1000)
}

// ── CAPHELD ───────────────────────────────────────────────────────────────────

#[test]
fn capheld_returns_true_for_registered_cap() {
    let cap_id = Hash::of(b"test_cap_a");
    let mut reg = CapabilityRegistry::new();
    reg.register(cap_id, Box::new(StubCap));
    let arc = reg.into_arc();

    let mut store = SubstanceStore::new();
    let verdict = run_with_capreg(
        Opcode::CapHeld,
        vec![Value::Hash(cap_id)],
        &mut store,
        arc,
    );
    assert!(
        matches!(verdict, ExecVerdict::Returned(Value::Bool(true))),
        "CAPHELD must return true for a registered cap; got {:?}",
        verdict
    );
}

#[test]
fn capheld_returns_false_for_unknown_cap() {
    let known = Hash::of(b"test_cap_b");
    let unknown = Hash::of(b"not_registered");
    let mut reg = CapabilityRegistry::new();
    reg.register(known, Box::new(StubCap));
    let arc = reg.into_arc();

    let mut store = SubstanceStore::new();
    let verdict = run_with_capreg(
        Opcode::CapHeld,
        vec![Value::Hash(unknown)],
        &mut store,
        arc,
    );
    assert!(
        matches!(verdict, ExecVerdict::Returned(Value::Bool(false))),
        "CAPHELD must return false for an unregistered cap"
    );
}

#[test]
fn capheld_returns_false_with_no_registry() {
    let cap_id = Hash::of(b"any_cap");
    let mut store = SubstanceStore::new();
    let verdict = run_single(Opcode::CapHeld, vec![Value::Hash(cap_id)], &mut store);
    assert!(
        matches!(verdict, ExecVerdict::Returned(Value::Bool(false))),
        "CAPHELD with no cap registry must return false (no caps held)"
    );
}

// ── ATTENUATE ─────────────────────────────────────────────────────────────────

#[test]
fn attenuate_returns_new_hash_when_cap_is_held() {
    let cap_id = Hash::of(b"parent_cap");
    let predicate = Value::Nat(42);

    let mut reg = CapabilityRegistry::new();
    reg.register(cap_id, Box::new(StubCap));
    let arc = reg.into_arc();

    let mut store = SubstanceStore::new();
    // IL convention: cap_id pushed first, predicate pushed second (predicate on top).
    // ExecState::new puts inputs[0] on top, so inputs = [predicate, cap_id].
    let verdict = run_with_capreg(
        Opcode::Attenuate,
        vec![predicate, Value::Hash(cap_id)],
        &mut store,
        arc,
    );
    match verdict {
        ExecVerdict::Returned(Value::Hash(_new_cap)) => {
            // Success: a new CapId hash was produced.
        }
        other => panic!("ATTENUATE expected Returned(Hash), got {:?}", other),
    }
}

#[test]
fn attenuate_traps_enotheld_when_cap_not_held() {
    let cap_id = Hash::of(b"unregistered_cap");
    let predicate = Value::Nat(0);

    let mut store = SubstanceStore::new();
    // inputs[0] = predicate (top), inputs[1] = cap_id (bottom)
    let verdict = run_single(
        Opcode::Attenuate,
        vec![predicate, Value::Hash(cap_id)],
        &mut store,
    );
    assert!(
        matches!(verdict, ExecVerdict::Trapped(TrapKind::ENotHeld)),
        "ATTENUATE must trap ENotHeld when cap is not held"
    );
}

#[test]
fn attenuate_new_hash_differs_from_original() {
    let cap_id = Hash::of(b"original_cap");
    let predicate = Value::Nat(99);

    let mut reg = CapabilityRegistry::new();
    reg.register(cap_id, Box::new(StubCap));
    let arc = reg.into_arc();

    let mut store = SubstanceStore::new();
    let verdict = run_with_capreg(
        Opcode::Attenuate,
        vec![predicate, Value::Hash(cap_id)],
        &mut store,
        arc,
    );
    match verdict {
        ExecVerdict::Returned(Value::Hash(new_cap)) => {
            assert_ne!(
                new_cap, cap_id,
                "ATTENUATE must produce a new CapId distinct from the original"
            );
        }
        other => panic!("expected Hash, got {:?}", other),
    }
}

// ── REVOKE ────────────────────────────────────────────────────────────────────

#[test]
fn revoke_succeeds_when_cap_is_held() {
    let cap_id = Hash::of(b"revoke_me");
    let mut reg = CapabilityRegistry::new();
    reg.register(cap_id, Box::new(StubCap));
    let arc = reg.into_arc();

    let mut store = SubstanceStore::new();
    // REVOKE pops CapId and returns (); RET pops the Unit left on stack.
    // We push Unit explicitly as the "return value" since REVOKE itself
    // doesn't push anything (type () → ()).
    // Build a custom program: PUSH unit, REVOKE-arg setup, REVOKE, RET.
    // Simpler: use a 3-instruction program: PUSH cap_id, REVOKE, RET.
    let code = vec![
        Opcode::Push(Value::Hash(cap_id)),
        Opcode::Revoke,
        Opcode::Push(Value::Unit), // leave something for RET to pop
        Opcode::Ret,
    ];
    let mut state = ExecState::new(Hash::BOTTOM, code, 0, vec![]);
    let mut interp = Interpreter::new(&mut store).with_cap_registry(arc);
    let verdict = interp.run(&mut state, 100);
    assert!(
        matches!(verdict, ExecVerdict::Returned(Value::Unit)),
        "REVOKE on a held cap must succeed; got {:?}",
        verdict
    );
}

#[test]
fn revoke_traps_enotheld_when_cap_not_held() {
    let cap_id = Hash::of(b"no_such_cap");
    let mut store = SubstanceStore::new();
    let verdict = run_single(Opcode::Revoke, vec![Value::Hash(cap_id)], &mut store);
    assert!(
        matches!(verdict, ExecVerdict::Trapped(TrapKind::ENotHeld)),
        "REVOKE must trap ENotHeld when cap is not held"
    );
}

// ── APPEND ────────────────────────────────────────────────────────────────────

#[test]
fn append_always_traps_estale_at_stage0() {
    let entry_hash = Hash::of(b"some_entry");
    let mut store = SubstanceStore::new();
    let verdict = run_single(Opcode::Append, vec![Value::Hash(entry_hash)], &mut store);
    assert!(
        matches!(verdict, ExecVerdict::Trapped(TrapKind::EStale)),
        "APPEND must trap EStale at stage-0 (no weave log)"
    );
}

// ── WHY ───────────────────────────────────────────────────────────────────────

#[test]
fn why_returns_empty_vec_at_stage0() {
    let sub_hash = Hash::of(b"some_substance");
    let mut store = SubstanceStore::new();
    let verdict = run_single(Opcode::Why, vec![Value::Hash(sub_hash)], &mut store);
    assert!(
        matches!(verdict, ExecVerdict::Returned(Value::Vec(ref v)) if v.is_empty()),
        "WHY must return empty Vec at stage-0 (no weave log); got {:?}",
        verdict
    );
}

// ── SPLIT ─────────────────────────────────────────────────────────────────────

#[test]
fn split_always_traps_eoverbudget_at_stage0() {
    let budget = Value::Nat(100);
    let mut store = SubstanceStore::new();
    let verdict = run_single(Opcode::Split, vec![budget], &mut store);
    assert!(
        matches!(verdict, ExecVerdict::Trapped(TrapKind::EOverBudget)),
        "SPLIT must trap EOverBudget at stage-0 (no attention allocator)"
    );
}

// ── READSLOT ──────────────────────────────────────────────────────────────────

#[test]
fn readslot_returns_bound_form_hash() {
    let name_hash = Hash::of(b"S-07/parse_form");
    let form_hash = Hash::of(b"form_bytes_placeholder");

    let mut reg = FormRegistry::new();
    reg.bind_slot(name_hash, form_hash);

    let mut store = SubstanceStore::new();
    let verdict = run_with_registry(
        Opcode::ReadSlot,
        vec![Value::Hash(name_hash)],
        &mut store,
        &reg,
    );
    assert!(
        matches!(verdict, ExecVerdict::Returned(Value::Hash(h)) if h == form_hash),
        "READSLOT must return the bound form_hash; got {:?}",
        verdict
    );
}

#[test]
fn readslot_traps_eunheld_when_slot_unbound() {
    let name_hash = Hash::of(b"unbound_slot");
    let reg = FormRegistry::new();

    let mut store = SubstanceStore::new();
    let verdict = run_with_registry(
        Opcode::ReadSlot,
        vec![Value::Hash(name_hash)],
        &mut store,
        &reg,
    );
    assert!(
        matches!(verdict, ExecVerdict::Trapped(TrapKind::EUnheld(_))),
        "READSLOT must trap EUnheld for an unbound slot"
    );
}

#[test]
fn readslot_traps_eunheld_with_no_registry() {
    let name_hash = Hash::of(b"any_slot");
    let mut store = SubstanceStore::new();
    let verdict = run_single(Opcode::ReadSlot, vec![Value::Hash(name_hash)], &mut store);
    assert!(
        matches!(verdict, ExecVerdict::Trapped(TrapKind::EUnheld(_))),
        "READSLOT with no registry must trap EUnheld"
    );
}

// ── BINDSLOT ──────────────────────────────────────────────────────────────────

#[test]
fn bindslot_always_traps_eunauthorised_at_stage0() {
    let name_hash = Value::Hash(Hash::of(b"slot_name"));
    let form_hash = Value::Hash(Hash::of(b"slot_form"));

    let mut store = SubstanceStore::new();
    // IL convention: name_hash pushed first, form_hash pushed second (form_hash on top).
    // inputs[0] = form_hash (top), inputs[1] = name_hash.
    let verdict = run_single(
        Opcode::BindSlot,
        vec![form_hash, name_hash],
        &mut store,
    );
    assert!(
        matches!(verdict, ExecVerdict::Trapped(TrapKind::EUnauthorised)),
        "BINDSLOT must trap EUnauthorised at stage-0 (no kernel mutation cap)"
    );
}

// ── PARSEFORM ─────────────────────────────────────────────────────────────────

#[test]
fn parseform_returns_hash_for_valid_form_bytes() {
    // Build a minimal valid Form, encode to wire bytes, seal in the store,
    // then run PARSEFORM. Expect a Returned(Hash(_)) for ParsedForm/v1.
    let form = Form {
        type_tag: "Form/v1".to_string(),
        arity: 0,
        locals_n: 0,
        declared_caps: vec![],
        declared_traps: vec![],
        code: vec![Opcode::Push(Value::Unit), Opcode::Ret],
    };
    let bytes = encode_form(&form).expect("encode_form must succeed for a valid form");

    let mut store = SubstanceStore::new();
    let h = store.seal("Bytes/v1", Value::Bytes(bytes));

    let verdict = run_single(Opcode::ParseForm, vec![Value::Hash(h)], &mut store);
    match verdict {
        ExecVerdict::Returned(Value::Hash(_parsed_hash)) => {
            // A ParsedForm/v1 substance hash was returned — success.
        }
        other => panic!("PARSEFORM expected Returned(Hash), got {:?}", other),
    }
}

#[test]
fn parseform_traps_etype_for_non_form_bytes() {
    let mut store = SubstanceStore::new();
    let garbage = Value::Bytes(b"not a form at all".to_vec());
    let h = store.seal("Bytes/v1", garbage);

    let verdict = run_single(Opcode::ParseForm, vec![Value::Hash(h)], &mut store);
    assert!(
        matches!(verdict, ExecVerdict::Trapped(TrapKind::EType(_))),
        "PARSEFORM must trap EType for invalid bytes"
    );
}

#[test]
fn parseform_traps_etype_for_non_bytes_substance() {
    let mut store = SubstanceStore::new();
    // Seal a Nat (not Bytes) and try to PARSEFORM it.
    let h = store.seal("Nat/v1", Value::Nat(42));

    let verdict = run_single(Opcode::ParseForm, vec![Value::Hash(h)], &mut store);
    assert!(
        matches!(verdict, ExecVerdict::Trapped(TrapKind::EType(_))),
        "PARSEFORM must trap EType when the substance is not Bytes"
    );
}

#[test]
fn parseform_result_hash_differs_from_input_hash() {
    // The ParsedForm/v1 substance has a different tag than the original
    // Bytes/v1 substance, so its hash must differ.
    let form = Form {
        type_tag: "Form/v1".to_string(),
        arity: 1,
        locals_n: 1,
        declared_caps: vec![],
        declared_traps: vec![],
        code: vec![Opcode::Load(0), Opcode::Ret],
    };
    let bytes = encode_form(&form).expect("encode failed");
    let mut store = SubstanceStore::new();
    let input_hash = store.seal("Bytes/v1", Value::Bytes(bytes));

    let verdict = run_single(Opcode::ParseForm, vec![Value::Hash(input_hash)], &mut store);
    match verdict {
        ExecVerdict::Returned(Value::Hash(parsed_hash)) => {
            assert_ne!(
                parsed_hash, input_hash,
                "ParsedForm/v1 hash must differ from the original Bytes/v1 hash"
            );
        }
        other => panic!("expected Hash, got {:?}", other),
    }
}

// ── CALLI ─────────────────────────────────────────────────────────────────────

/// Build a minimal callee Form that takes one Nat on the stack (as arg₀),
/// adds 1, and returns. The caller invokes it via CALLI to exercise the
/// indirect-call path end-to-end.
fn register_increment_form(reg: &mut FormRegistry) -> Hash {
    // Code: STORE 0; LOAD 0; PUSH 1; ADD; RET
    // Arity: 1, locals_n: 1 — identical body to the A9.3 canonical F.
    let callee = LoadedForm {
        name: "increment".to_string(),
        locals_n: 1,
        code: vec![
            Opcode::Store(0),
            Opcode::Load(0),
            Opcode::Push(Value::Nat(1)),
            Opcode::Add,
            Opcode::Ret,
        ],
    };
    // Dispatch is by content hash; at stage-0 the choice of hash
    // doesn't need to match canonical wire bytes because nothing
    // else in this test reaches back into the store. Pick a
    // deterministic hash so the binding-side code matches.
    let h = Hash::of(b"ignis0/test/increment");
    reg.register(h, callee);
    h
}

#[test]
fn calli_dispatches_to_stack_top_hash() {
    // Direct form of the idiom: PUSH arg; PUSH form_hash; CALLI /n=1; RET.
    // No READSLOT involved — pins CALLI's contract (consume top hash,
    // invoke it with n args) independently of slot resolution.
    let mut reg = FormRegistry::new();
    let target = register_increment_form(&mut reg);

    let mut store = SubstanceStore::new();
    let code = vec![
        Opcode::Push(Value::Nat(42)),
        Opcode::Push(Value::Hash(target)),
        Opcode::CallI { n: 1 },
        Opcode::Ret,
    ];
    let mut state = ExecState::new(Hash::BOTTOM, code, 0, vec![]);
    let mut interp = Interpreter::new(&mut store).with_registry(&reg);
    let verdict = interp.run(&mut state, 100);
    assert!(
        matches!(verdict, ExecVerdict::Returned(Value::Nat(43))),
        "CALLI must dispatch to the stack-top hash (42 + 1 = 43); got {:?}",
        verdict
    );
}

#[test]
fn readslot_plus_calli_resolves_and_invokes() {
    // The canonical idiom from IL.md § Control flow: the whole point of
    // the 34→35 bump. PUSH name; READSLOT; CALLI /n=1 must chain so the
    // target Form bound at `name` runs with the supplied arg.
    let mut reg = FormRegistry::new();
    let target = register_increment_form(&mut reg);
    let slot_name = Hash::of(b"test/slot/increment");
    reg.bind_slot(slot_name, target);

    let mut store = SubstanceStore::new();
    let code = vec![
        Opcode::Push(Value::Nat(99)),
        Opcode::Push(Value::Hash(slot_name)),
        Opcode::ReadSlot,
        Opcode::CallI { n: 1 },
        Opcode::Ret,
    ];
    let mut state = ExecState::new(Hash::BOTTOM, code, 0, vec![]);
    let mut interp = Interpreter::new(&mut store).with_registry(&reg);
    let verdict = interp.run(&mut state, 100);
    assert!(
        matches!(verdict, ExecVerdict::Returned(Value::Nat(100))),
        "READSLOT + CALLI must compose into slot dispatch (99 + 1 = 100); got {:?}",
        verdict
    );
}

#[test]
fn calli_traps_etype_on_non_hash_top() {
    // CALLI expects a Hash on top of the stack. A Nat must produce ETYPE.
    let reg = FormRegistry::new();
    let mut store = SubstanceStore::new();
    let code = vec![
        Opcode::Push(Value::Nat(7)),
        Opcode::CallI { n: 0 },
        Opcode::Ret,
    ];
    let mut state = ExecState::new(Hash::BOTTOM, code, 0, vec![]);
    let mut interp = Interpreter::new(&mut store).with_registry(&reg);
    let verdict = interp.run(&mut state, 100);
    assert!(
        matches!(verdict, ExecVerdict::Trapped(TrapKind::EType(_))),
        "CALLI must trap ETYPE when the top of stack is not a Hash; got {:?}",
        verdict
    );
}

#[test]
fn calli_traps_eunheld_for_unregistered_hash() {
    // CALLI resolves via the same registry path as direct CALL, so an
    // unknown hash must surface EUNHELD.
    let reg = FormRegistry::new();
    let unknown = Hash::of(b"no-such-form");
    let mut store = SubstanceStore::new();
    let code = vec![
        Opcode::Push(Value::Hash(unknown)),
        Opcode::CallI { n: 0 },
        Opcode::Ret,
    ];
    let mut state = ExecState::new(Hash::BOTTOM, code, 0, vec![]);
    let mut interp = Interpreter::new(&mut store).with_registry(&reg);
    let verdict = interp.run(&mut state, 100);
    assert!(
        matches!(verdict, ExecVerdict::Trapped(TrapKind::EUnheld(_))),
        "CALLI must trap EUNHELD when the resolved hash is not registered; got {:?}",
        verdict
    );
}
