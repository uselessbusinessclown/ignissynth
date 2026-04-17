//! Form Envelope — the derivation-gating control plane.
//!
//! A `FormEnvelope` is a JSON-shaped wrapper around a payload that
//! carries everything an external verifier needs to decide whether the
//! payload may execute, and under what restrictions. It sits *above*
//! the IL interpreter (`exec::Interpreter`): the IL knows about
//! opcodes, stacks, traps; the envelope knows about derivation, proof
//! status, declared capabilities, and parent linkage.
//!
//! This module is a deliberately small demo of the principle that
//! ignition is gated by *structure and derivation*, not just by
//! syntactic well-formedness. The full apparatus — the breakdown
//! chain in `breakdown/`, the proof artifacts under
//! `kernel/forms/S-*.proof`, the eventual `S-08` checker — is what
//! this layer is a stand-in for. The spec-level design is in
//! `kernel/IL.md` (`:declared-caps`, `:rationale-hash`, etc.); this
//! module surfaces the same idea at a JSON-friendly external surface
//! so it can be exercised end-to-end before the spec layer is live.
//!
//! ## Hash discipline
//!
//! The envelope's `hash` field is BLAKE3 (hex) of the *canonical JSON
//! encoding of the envelope with its `hash` field stripped*. Canonical
//! here means: serde struct-field order (declaration order), arrays as
//! given, no whitespace. `parents`, `capabilities`, and
//! `open_obligations` are required to be lexicographically sorted on
//! disk so that semantically identical envelopes hash identically.
//!
//! ## Position relative to the IL
//!
//! `FormEnvelope::Op` is an envelope-level operation, distinct from
//! the IL `Opcode`. The two should not be confused. An envelope
//! payload could in principle carry a sealed IL Form hash, but for
//! the demo the payload is a tiny effect language that lets the three
//! execution modes (Full / Restricted / Denied) be exercised without
//! pulling in the full IL stack.

use serde::{Deserialize, Serialize};
use thiserror::Error;

// ── Well-known capability names ──────────────────────────────────────
//
// The envelope-level capabilities are strings the payload's ops require
// and the envelope declares. Keeping them as constants lets callers
// (tests, CLI, derived forms) reference the same spelling, and removes
// the stringly-typed foot-gun of a silent cap-name typo passing verify.

/// Capability required by `Op::Write`. A declared `io.fs` is the
/// envelope-level equivalent of an IL `:declared-caps` entry that
/// grants filesystem write authority.
pub const IO_FS_CAP: &str = "io.fs";

/// Capability required by `Op::Infer`. Envelope-level analogue of the
/// IL `Synthesis/infer/v1` cap.
pub const INFER_REMOTE_CAP: &str = "infer.remote";

/// Outcome of the (eventual) proof checker for a given Form.
///
/// In the full system this is the verdict produced by `S-08` over the
/// Form's proof artifact; here it is a coarse three-valued tag the
/// envelope carries explicitly. The runner reads this tag to decide
/// which execution mode applies (see `crate::runner::EnvelopeMode`).
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ProofStatus {
    /// Proof discharged. Full execution allowed (subject to capability
    /// checks).
    Verified,
    /// Proof not yet checked or has open obligations. The Form may run
    /// only in *restricted* mode — observable-only operations.
    Deferred,
    /// Proof checked and rejected. Execution is denied.
    Invalid,
}

/// One operation in an envelope payload.
///
/// Distinct from `crate::Opcode` (the IL). These are envelope-level
/// effects whose only purpose is to make the three execution modes
/// concretely observable in the demo. The capability requirements are
/// declared per-variant in `Op::required_capability`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "op", rename_all = "lowercase")]
pub enum Op {
    /// Print a string. Observable-only — no capability required, and
    /// allowed even in restricted execution mode.
    Emit { text: String },
    /// Write a string to a file path. Requires the `io.fs` capability
    /// and is denied in restricted mode.
    Write { path: String, content: String },
    /// Submit a prompt to a remote inference endpoint. Requires the
    /// `infer.remote` capability and is denied in restricted mode.
    Infer { prompt: String },
}

impl Op {
    /// The capability required to execute this op, if any. `None`
    /// means the op is observable-only and may execute under any mode
    /// that allows execution at all (including restricted).
    pub fn required_capability(&self) -> Option<&'static str> {
        match self {
            Op::Emit { .. } => None,
            Op::Write { .. } => Some(IO_FS_CAP),
            Op::Infer { .. } => Some(INFER_REMOTE_CAP),
        }
    }

    /// Whether this op is observable-only — no external side effects
    /// beyond producing log lines. The runner's restricted mode
    /// permits exactly the observable-only ops.
    pub fn is_observable_only(&self) -> bool {
        matches!(self, Op::Emit { .. })
    }

    /// Short lowercase tag used in structured output and diagnostics.
    /// Mirrors `Opcode::mnemonic` for the IL; matches the serde
    /// `rename_all = "lowercase"` tag on `Op`.
    pub fn name(&self) -> &'static str {
        match self {
            Op::Emit { .. } => "emit",
            Op::Write { .. } => "write",
            Op::Infer { .. } => "infer",
        }
    }
}

/// The execution payload carried by an envelope.
///
/// A list of ops executed in declaration order. The runner decides per
/// op whether to permit, log-and-skip, or refuse the entire payload
/// based on the envelope's proof status and declared capabilities.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Payload {
    pub ops: Vec<Op>,
}

/// The derivation-gating envelope around a payload.
///
/// Field order is significant: it determines the canonical JSON
/// encoding used for hashing. Do not reorder without bumping the
/// envelope version (today: implicit v1).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct FormEnvelope {
    /// Stable, human-readable identifier. By convention derived forms
    /// use `<parent_id>:<rule>:<short-hash>`; genesis forms use any
    /// caller-chosen string.
    pub form_id: String,

    /// BLAKE3 (lowercase hex, 64 chars) of the envelope's canonical
    /// JSON with this field stripped. The verifier recomputes this
    /// and rejects on mismatch. See `compute_canonical_hash`.
    pub hash: String,

    /// Form ids of immediate predecessors. Empty iff `rule ==
    /// "genesis"` (otherwise the form is an orphan and is rejected).
    /// Must be lexicographically sorted on disk.
    pub parents: Vec<String>,

    /// The derivation rule that produced this form from its parents.
    /// The sentinel `"genesis"` marks a form with no predecessors.
    pub rule: String,

    /// Coarse stand-in for the proof checker verdict. Drives the
    /// execution mode at runtime.
    pub proof_status: ProofStatus,

    /// Obligations that are not yet discharged. A `verified` form is
    /// allowed to carry this list non-empty (some downstream checks
    /// may legitimately defer), but the verifier reports them so a
    /// reviewer can decide. Must be sorted on disk.
    pub open_obligations: Vec<String>,

    /// Capabilities the form *declares* it will need. The runner
    /// refuses any op whose required capability is not in this list,
    /// regardless of proof status. Must be sorted on disk.
    pub capabilities: Vec<String>,

    /// What the form does when executed.
    pub payload: Payload,
}

/// The set of fields that participate in the canonical hash.
///
/// This mirrors `FormEnvelope` minus the `hash` field. Keeping the
/// field order in lockstep with `FormEnvelope` is what makes the
/// canonical hash stable and reproducible across implementations.
///
/// Implementation note: we manually mirror the struct rather than
/// using a serde `skip` attribute because we want the ABI guarantee
/// that adding a new field to `FormEnvelope` is a breaking change to
/// the hash, surfaced at compile time when this struct is updated.
#[derive(Serialize)]
struct CanonicalView<'a> {
    form_id: &'a str,
    parents: &'a [String],
    rule: &'a str,
    proof_status: ProofStatus,
    open_obligations: &'a [String],
    capabilities: &'a [String],
    payload: &'a Payload,
}

impl FormEnvelope {
    /// Compute the canonical BLAKE3 hash of this envelope (hex,
    /// 64 chars). Independent of the value of `self.hash`.
    pub fn compute_canonical_hash(&self) -> String {
        let view = CanonicalView {
            form_id: &self.form_id,
            parents: &self.parents,
            rule: &self.rule,
            proof_status: self.proof_status,
            open_obligations: &self.open_obligations,
            capabilities: &self.capabilities,
            payload: &self.payload,
        };
        // serde_json with no whitespace and struct-declaration field
        // order is our canonical encoding.
        let bytes = serde_json::to_vec(&view).expect("canonical envelope view must serialize");
        blake3::hash(&bytes).to_hex().to_string()
    }

    /// Produce a copy with `hash` overwritten to the canonical value.
    /// Useful when constructing or deriving an envelope.
    pub fn with_canonical_hash(mut self) -> Self {
        self.hash = self.compute_canonical_hash();
        self
    }

    /// True iff this envelope claims to be a genesis form (no
    /// derivation predecessors). Genesis forms are allowed to have
    /// `parents == []`; non-genesis forms with empty parents are
    /// orphans and are rejected.
    pub fn is_genesis(&self) -> bool {
        self.rule == GENESIS_RULE && self.parents.is_empty()
    }

    /// Parse from JSON bytes. Returns a structured error so the
    /// `explain` command can report parse failures cleanly.
    pub fn from_json_bytes(bytes: &[u8]) -> Result<Self, EnvelopeParseError> {
        serde_json::from_slice(bytes).map_err(|e| EnvelopeParseError(e.to_string()))
    }

    /// Serialize to pretty JSON suitable for writing to disk by hand.
    /// The sort order of `parents`/`capabilities`/`open_obligations`
    /// is the caller's responsibility; the runner's tests build them
    /// sorted.
    pub fn to_pretty_json(&self) -> String {
        serde_json::to_string_pretty(self).expect("envelope must serialize")
    }
}

/// Reserved derivation-rule sentinel for genesis forms.
pub const GENESIS_RULE: &str = "genesis";

/// JSON parse error wrapped so callers can match without depending on
/// `serde_json` directly.
#[derive(Debug, Clone, Error)]
#[error("envelope JSON parse error: {0}")]
pub struct EnvelopeParseError(pub String);

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_genesis() -> FormEnvelope {
        FormEnvelope {
            form_id: "demo/hello".into(),
            hash: String::new(),
            parents: vec![],
            rule: GENESIS_RULE.into(),
            proof_status: ProofStatus::Verified,
            open_obligations: vec![],
            capabilities: vec![],
            payload: Payload {
                ops: vec![Op::Emit {
                    text: "hello, ignis".into(),
                }],
            },
        }
        .with_canonical_hash()
    }

    #[test]
    fn canonical_hash_is_reproducible() {
        let a = fixture_genesis();
        let b = fixture_genesis();
        assert_eq!(a.hash, b.hash);
        assert_eq!(a.hash.len(), 64, "hash must be 64 hex chars (BLAKE3-256)");
    }

    #[test]
    fn canonical_hash_ignores_self_hash_field() {
        let mut a = fixture_genesis();
        let original = a.hash.clone();
        a.hash = "deadbeef".into();
        // recomputing must yield the original — the hash field does
        // not contribute to its own input.
        assert_eq!(a.compute_canonical_hash(), original);
    }

    #[test]
    fn canonical_hash_changes_on_payload_edit() {
        let a = fixture_genesis();
        let mut b = fixture_genesis();
        b.payload.ops.push(Op::Emit {
            text: "second line".into(),
        });
        assert_ne!(a.compute_canonical_hash(), b.compute_canonical_hash());
    }

    #[test]
    fn op_required_capability_table() {
        assert_eq!(Op::Emit { text: "x".into() }.required_capability(), None);
        assert_eq!(
            Op::Write {
                path: "/tmp/x".into(),
                content: "y".into()
            }
            .required_capability(),
            Some("io.fs")
        );
        assert_eq!(
            Op::Infer { prompt: "p".into() }.required_capability(),
            Some("infer.remote")
        );
    }

    #[test]
    fn round_trip_through_json() {
        let a = fixture_genesis();
        let json = a.to_pretty_json();
        let b = FormEnvelope::from_json_bytes(json.as_bytes()).unwrap();
        assert_eq!(a, b);
        assert_eq!(b.hash, b.compute_canonical_hash());
    }

    #[test]
    fn is_genesis_requires_both_rule_and_empty_parents() {
        let g = fixture_genesis();
        assert!(g.is_genesis());

        let mut not_genesis_rule = g.clone();
        not_genesis_rule.rule = "step".into();
        assert!(!not_genesis_rule.is_genesis());

        let mut not_genesis_parents = g.clone();
        not_genesis_parents.parents = vec!["x".into()];
        assert!(!not_genesis_parents.is_genesis());
    }
}
