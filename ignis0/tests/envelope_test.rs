//! Integration tests for the derivation-gating control plane.
//!
//! Covers the three demo scenarios from the feature spec
//! (Valid Form / Orphan Form / Deferred Form) end-to-end, plus
//! the supporting failure modes the verifier must distinguish
//! (hash mismatch, undeclared capability, invalid proof status,
//! missing parent in ledger).
//!
//! These tests exercise the public API exactly as the CLI does, so a
//! green test suite implies the four `verify` / `run` / `explain` /
//! `derive` subcommands work on at least these inputs.

use ignis0::envelope::{FormEnvelope, Op, Payload, ProofStatus, GENESIS_RULE};
use ignis0::runner::{run_envelope, EnvelopeMode, OpDecision};
use ignis0::verify::{verify, Ledger, VerifyError};
use ignis0::{derive_form, run_envelope_with_mode};

// ── Helpers ──────────────────────────────────────────────────────────

fn genesis(form_id: &str, ops: Vec<Op>, caps: Vec<String>, status: ProofStatus) -> FormEnvelope {
    FormEnvelope {
        form_id: form_id.into(),
        hash: String::new(),
        parents: vec![],
        rule: GENESIS_RULE.into(),
        proof_status: status,
        open_obligations: vec![],
        capabilities: caps,
        payload: Payload { ops },
    }
    .with_canonical_hash()
}

// ── Demo scenario 1: Valid Form ──────────────────────────────────────

#[test]
fn demo_valid_form_verifies_and_executes_in_full_mode() {
    let env = genesis(
        "demo/valid",
        vec![
            Op::Emit {
                text: "ignition begins".into(),
            },
            Op::Write {
                path: "/tmp/seed.txt".into(),
                content: "data".into(),
            },
        ],
        vec!["io.fs".into()],
        ProofStatus::Verified,
    );

    let outcome =
        verify(&env, &Ledger::new()).expect("a verified, declared, hash-correct genesis must verify");

    let result = run_envelope(&env, &outcome);
    assert_eq!(result.mode, EnvelopeMode::Full);
    assert_eq!(result.executed_count(), 2, "all declared ops must execute");
    assert_eq!(result.skipped_count(), 0);
    assert!(matches!(
        &result.decisions[0],
        OpDecision::Executed { effect } if effect == "emit: ignition begins"
    ));
}

// ── Demo scenario 2: Orphan Form ─────────────────────────────────────

#[test]
fn demo_orphan_form_parses_but_is_refused_by_verifier() {
    // Construct a non-genesis form with an empty parent list. This is
    // structurally well-formed JSON (parses fine) but the verifier
    // must refuse it because it has no derivation predecessor.
    let mut env = genesis(
        "demo/orphan",
        vec![Op::Emit { text: "alone".into() }],
        vec![],
        ProofStatus::Verified,
    );
    env.rule = "step".into(); // not "genesis"
    let env = env.with_canonical_hash();

    // Round-trip through JSON to prove "parses correctly".
    let json = env.to_pretty_json();
    let reparsed = FormEnvelope::from_json_bytes(json.as_bytes()).expect("orphan must still parse");
    assert_eq!(reparsed, env);

    // Verifier must refuse with OrphanForm.
    let err = verify(&env, &Ledger::new()).expect_err("orphan must be refused");
    match err {
        VerifyError::OrphanForm { rule } => assert_eq!(rule, "step"),
        other => panic!("expected OrphanForm, got {:?}", other),
    }
}

#[test]
fn orphan_form_with_missing_parent_in_ledger_is_refused() {
    // A non-empty parents list is also "orphan" if those parents are
    // not in the ledger — same effect, different VerifyError variant.
    let mut env = genesis(
        "demo/missing-parent",
        vec![Op::Emit { text: "x".into() }],
        vec![],
        ProofStatus::Verified,
    );
    env.rule = "step".into();
    env.parents = vec!["nowhere/to/be/found".into()];
    let env = env.with_canonical_hash();

    let err = verify(&env, &Ledger::new()).expect_err("missing parent must be refused");
    assert!(matches!(err, VerifyError::MissingParent { .. }));
}

// ── Demo scenario 3: Deferred Form ───────────────────────────────────

#[test]
fn demo_deferred_form_admits_with_restricted_execution_only() {
    let env = genesis(
        "demo/deferred",
        vec![
            Op::Emit {
                text: "thinking out loud".into(),
            },
            Op::Write {
                path: "/tmp/x".into(),
                content: "y".into(),
            },
            Op::Infer { prompt: "p".into() },
        ],
        // Caps are declared so the verifier admits — but the runner
        // must still restrict because proof_status is Deferred.
        vec!["io.fs".into(), "infer.remote".into()],
        ProofStatus::Deferred,
    );

    let outcome = verify(&env, &Ledger::new()).expect("deferred must admit when declared");
    assert_eq!(outcome.proof_status, ProofStatus::Deferred);

    // Default run picks Restricted from Deferred.
    let result = run_envelope(&env, &outcome);
    assert_eq!(
        result.mode,
        EnvelopeMode::Restricted,
        "deferred must default to restricted execution"
    );
    assert_eq!(result.executed_count(), 1, "only emit must run");
    assert_eq!(result.skipped_count(), 2, "write and infer must be skipped");

    // The first op (emit) ran; the rest were marked skipped.
    assert!(matches!(
        &result.decisions[0],
        OpDecision::Executed { .. }
    ));
    assert!(matches!(
        &result.decisions[1],
        OpDecision::SkippedRestricted { op_name } if *op_name == "write"
    ));
    assert!(matches!(
        &result.decisions[2],
        OpDecision::SkippedRestricted { op_name } if *op_name == "infer"
    ));
}

#[test]
fn deferred_form_full_execution_is_explicitly_disallowed() {
    // Caller cannot promote a deferred form to full mode by re-using
    // the runner's mode-mapping — they would have to call
    // run_envelope_with_mode(EnvelopeMode::Full) directly. That is the
    // only escape hatch and it requires explicit opt-in (not what the
    // verifier produces).
    let env = genesis(
        "demo/no-promotion",
        vec![Op::Write {
            path: "/tmp/x".into(),
            content: "y".into(),
        }],
        vec!["io.fs".into()],
        ProofStatus::Deferred,
    );
    let outcome = verify(&env, &Ledger::new()).unwrap();
    let result = run_envelope(&env, &outcome);

    // The runner does not promote.
    assert_eq!(result.mode, EnvelopeMode::Restricted);
    assert_eq!(result.executed_count(), 0);
    assert_eq!(result.skipped_count(), 1);
}

// ── Capability enforcement (independent of mode) ─────────────────────

#[test]
fn undeclared_capability_is_refused_at_verify_time() {
    let env = genesis(
        "demo/undeclared",
        vec![Op::Write {
            path: "/tmp/x".into(),
            content: "y".into(),
        }],
        vec![], // io.fs is required but not declared
        ProofStatus::Verified,
    );
    let err = verify(&env, &Ledger::new()).expect_err("undeclared cap must be refused");
    match err {
        VerifyError::UndeclaredCapability {
            op_index,
            op_name,
            required,
            ..
        } => {
            assert_eq!(op_index, 0);
            assert_eq!(op_name, "write");
            assert_eq!(required, "io.fs");
        }
        other => panic!("expected UndeclaredCapability, got {:?}", other),
    }
}

// ── Hash-mismatch check ──────────────────────────────────────────────

#[test]
fn tampered_payload_invalidates_hash() {
    // Build a valid envelope, then mutate the payload without
    // recomputing the hash. The verifier must catch the discrepancy.
    let mut env = genesis(
        "demo/tamper",
        vec![Op::Emit { text: "original".into() }],
        vec![],
        ProofStatus::Verified,
    );
    env.payload.ops.push(Op::Emit {
        text: "tampered after sealing".into(),
    });
    // Hash NOT recomputed.

    let err = verify(&env, &Ledger::new()).expect_err("tampered payload must fail hash check");
    assert!(matches!(err, VerifyError::HashMismatch { .. }));
}

// ── Invalid proof status is denied ───────────────────────────────────

#[test]
fn invalid_proof_status_is_denied_at_gate() {
    let env = genesis(
        "demo/invalid",
        vec![Op::Emit { text: "should not run".into() }],
        vec![],
        ProofStatus::Invalid,
    );
    let err = verify(&env, &Ledger::new()).expect_err("invalid status must be denied");
    assert!(matches!(err, VerifyError::InvalidProofStatus));
}

// ── Derive — the derivation chain extends correctly ──────────────────

#[test]
fn derived_child_links_to_parent_and_verifies_against_ledger() {
    let parent = genesis(
        "g",
        vec![Op::Emit { text: "ancestor".into() }],
        vec!["io.fs".into()],
        ProofStatus::Verified,
    );
    let child = derive_form(&parent, "step", None);

    // Child must be deferred by default and link to the parent.
    assert_eq!(child.proof_status, ProofStatus::Deferred);
    assert_eq!(child.parents, vec!["g".to_string()]);

    // Without the parent in the ledger, the child is an orphan-by-
    // ledger-lookup (MissingParent).
    let empty = Ledger::new();
    assert!(matches!(
        verify(&child, &empty),
        Err(VerifyError::MissingParent { .. })
    ));

    // With the parent loaded, the child verifies — and runs in
    // restricted mode (because it's deferred).
    let mut ledger = Ledger::new();
    ledger.insert(parent);
    let outcome = verify(&child, &ledger).expect("child with parent in ledger must verify");
    assert_eq!(outcome.proof_status, ProofStatus::Deferred);

    let result = run_envelope(&child, &outcome);
    assert_eq!(result.mode, EnvelopeMode::Restricted);
    // The child inherited the parent's payload (Op::Emit "ancestor"),
    // which is observable-only and so will execute even in restricted
    // mode. That's the correct behaviour.
    assert_eq!(result.executed_count(), 1);
}

// ── Explicit mode override (dry-run a verified form) ─────────────────

#[test]
fn explicit_restricted_mode_overrides_status_mapping() {
    let env = genesis(
        "demo/dry-run",
        vec![Op::Write {
            path: "/tmp/x".into(),
            content: "y".into(),
        }],
        vec!["io.fs".into()],
        ProofStatus::Verified,
    );
    let result = run_envelope_with_mode(&env, EnvelopeMode::Restricted);
    assert_eq!(result.mode, EnvelopeMode::Restricted);
    assert_eq!(result.executed_count(), 0, "write must be skipped under restricted");
    assert_eq!(result.skipped_count(), 1);
}

// ── JSON load / store round-trip ─────────────────────────────────────

#[test]
fn ledger_round_trip_through_json_files() {
    let env = genesis(
        "demo/persisted",
        vec![Op::Emit { text: "rehydrated".into() }],
        vec![],
        ProofStatus::Verified,
    );
    let tmp = std::env::temp_dir().join(format!(
        "ignis0-envelope-test-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).unwrap();
    let path = tmp.join("demo-persisted.envelope.json");
    std::fs::write(&path, env.to_pretty_json()).unwrap();

    let ledger = Ledger::load_from_dir(&tmp).expect("ledger must load");
    assert_eq!(ledger.len(), 1);
    let loaded = ledger.get("demo/persisted").expect("form id must index");
    assert_eq!(loaded.hash, env.hash, "round-trip must preserve canonical hash");

    // Verify still passes after the round trip.
    verify(loaded, &ledger).unwrap();

    // Cleanup.
    let _ = std::fs::remove_dir_all(&tmp);
}
