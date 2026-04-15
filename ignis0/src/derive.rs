//! Derive — produce a new envelope from an existing one.
//!
//! `derive_form` takes a parent envelope and a derivation rule id and
//! produces a child envelope with:
//!
//! - `parents = [parent.form_id]`
//! - `rule = <given>`
//! - `proof_status = Deferred` (children inherit the burden of proof)
//! - `open_obligations = []` (caller may extend)
//! - `capabilities = parent.capabilities` (children inherit declared
//!   caps as a starting point — if the new payload uses fewer, the
//!   caller may trim before sealing)
//! - `payload = parent.payload` (caller may overwrite via the
//!   `payload_override` argument)
//! - `form_id` derived from the parent's hash and the rule, so two
//!   independent derivations of the same parent under the same rule
//!   produce the same id (content-addressed derivation).
//! - `hash` recomputed canonically.
//!
//! The "deferred-by-default" rule is the conservative choice: a fresh
//! derivation has not been checked, so the runner restricts it to
//! observable-only ops. A reviewer must explicitly bump
//! `proof_status` to `Verified` after running the (eventual)
//! checker.

use crate::envelope::{FormEnvelope, Payload, ProofStatus};

/// Build a child envelope from a parent and a derivation rule.
///
/// `payload_override` lets the caller swap the payload (typical case);
/// pass `None` to inherit the parent's payload unchanged. The returned
/// envelope has its hash recomputed and is ready to be written to the
/// ledger.
pub fn derive_form(
    parent: &FormEnvelope,
    rule: &str,
    payload_override: Option<Payload>,
) -> FormEnvelope {
    assert!(
        rule != crate::envelope::GENESIS_RULE,
        "derive_form: cannot derive a genesis form (rule == \"genesis\" is reserved \
         for top-level forms with no parents)"
    );

    let payload = payload_override.unwrap_or_else(|| parent.payload.clone());
    let form_id = derive_id(&parent.hash, &parent.form_id, rule);

    FormEnvelope {
        form_id,
        hash: String::new(), // filled by with_canonical_hash below
        parents: vec![parent.form_id.clone()],
        rule: rule.to_string(),
        proof_status: ProofStatus::Deferred,
        open_obligations: Vec::new(),
        capabilities: parent.capabilities.clone(),
        payload,
    }
    .with_canonical_hash()
}

/// Compose a deterministic child id from `<parent_form_id>:<rule>:<short>`
/// where `short` is the first 8 hex chars of BLAKE3(parent_hash || rule).
/// Two derivations of the same parent under the same rule produce the
/// same id; differing rules (or parents) produce different ids.
fn derive_id(parent_hash: &str, parent_form_id: &str, rule: &str) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(parent_hash.as_bytes());
    hasher.update(b":");
    hasher.update(rule.as_bytes());
    let hex = hasher.finalize().to_hex();
    format!("{}:{}:{}", parent_form_id, rule, &hex[..8])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::envelope::{Op, GENESIS_RULE};

    fn parent() -> FormEnvelope {
        FormEnvelope {
            form_id: "g".into(),
            hash: String::new(),
            parents: vec![],
            rule: GENESIS_RULE.into(),
            proof_status: ProofStatus::Verified,
            open_obligations: vec![],
            capabilities: vec!["io.fs".into()],
            payload: Payload {
                ops: vec![Op::Emit { text: "x".into() }],
            },
        }
        .with_canonical_hash()
    }

    #[test]
    fn child_has_parent_id_in_parents() {
        let p = parent();
        let c = derive_form(&p, "step", None);
        assert_eq!(c.parents, vec!["g".to_string()]);
    }

    #[test]
    fn child_proof_status_defaults_to_deferred() {
        let p = parent();
        let c = derive_form(&p, "step", None);
        assert_eq!(c.proof_status, ProofStatus::Deferred);
    }

    #[test]
    fn child_inherits_capabilities() {
        let p = parent();
        let c = derive_form(&p, "step", None);
        assert_eq!(c.capabilities, p.capabilities);
    }

    #[test]
    fn child_id_is_deterministic() {
        let p = parent();
        let c1 = derive_form(&p, "step", None);
        let c2 = derive_form(&p, "step", None);
        assert_eq!(c1.form_id, c2.form_id);
        assert_eq!(c1.hash, c2.hash);
    }

    #[test]
    fn different_rules_produce_different_ids() {
        let p = parent();
        let c1 = derive_form(&p, "step", None);
        let c2 = derive_form(&p, "refine", None);
        assert_ne!(c1.form_id, c2.form_id);
    }

    #[test]
    fn child_hash_matches_canonical() {
        let p = parent();
        let c = derive_form(&p, "step", None);
        assert_eq!(c.hash, c.compute_canonical_hash());
    }

    #[test]
    fn payload_override_replaces_inherited_payload() {
        let p = parent();
        let new_payload = Payload {
            ops: vec![Op::Emit {
                text: "different".into(),
            }],
        };
        let c = derive_form(&p, "step", Some(new_payload.clone()));
        assert_eq!(c.payload, new_payload);
    }

    #[test]
    #[should_panic(expected = "cannot derive a genesis form")]
    fn cannot_derive_with_genesis_rule() {
        let p = parent();
        let _ = derive_form(&p, GENESIS_RULE, None);
    }
}
