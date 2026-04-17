//! Verifier — the gate between a parsed envelope and the runner.
//!
//! The verifier takes a `FormEnvelope` and a `Ledger` of known forms
//! and decides whether the envelope is *admissible*. Admission is
//! gated on six independent checks; each one can fail with a distinct
//! `VerifyError` variant so the `explain` command can report exactly
//! which gate refused.
//!
//! Admission says only that the envelope *may* be considered by the
//! runner. The runner's execution mode (Full / Restricted / Denied)
//! is then chosen from `proof_status`. So a deferred form may verify
//! and still be denied full execution downstream — that is by design.

use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};

use crate::envelope::{EnvelopeParseError, FormEnvelope, ProofStatus};

/// Where the verifier looks up parent form_ids.
///
/// In the demo this is a directory of `*.envelope.json` files; each
/// file is loaded once and indexed by its `form_id`. In the eventual
/// system this would be backed by `S-03` (the substance store).
#[derive(Debug, Default, Clone)]
pub struct Ledger {
    by_id: BTreeMap<String, FormEnvelope>,
}

impl Ledger {
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert a form. If a form with the same `form_id` is already in
    /// the ledger, it is replaced and the previous entry returned.
    pub fn insert(&mut self, env: FormEnvelope) -> Option<FormEnvelope> {
        self.by_id.insert(env.form_id.clone(), env)
    }

    pub fn contains(&self, form_id: &str) -> bool {
        self.by_id.contains_key(form_id)
    }

    pub fn get(&self, form_id: &str) -> Option<&FormEnvelope> {
        self.by_id.get(form_id)
    }

    pub fn len(&self) -> usize {
        self.by_id.len()
    }

    pub fn is_empty(&self) -> bool {
        self.by_id.is_empty()
    }

    /// Load every `*.envelope.json` under `dir` (non-recursive). Files
    /// that fail to parse are surfaced as `LedgerLoadError::ParseError`
    /// with the offending path so the caller can decide whether to
    /// continue or abort. This is intentionally strict; an unreadable
    /// or malformed envelope on the search path is treated as a real
    /// problem, not silently skipped.
    pub fn load_from_dir(dir: &Path) -> Result<Self, LedgerLoadError> {
        let mut ledger = Self::new();
        let entries = std::fs::read_dir(dir).map_err(|e| LedgerLoadError::Io {
            path: dir.to_path_buf(),
            source: e.to_string(),
        })?;
        for entry in entries {
            let entry = entry.map_err(|e| LedgerLoadError::Io {
                path: dir.to_path_buf(),
                source: e.to_string(),
            })?;
            // `file_type` avoids a second stat compared to `path.is_file()`
            // and gives us enough to skip subdirectories cheaply.
            let is_file = entry.file_type().map(|t| t.is_file()).unwrap_or(false);
            if !is_file {
                continue;
            }
            let path = entry.path();
            if !is_envelope_path(&path) {
                continue;
            }
            let bytes = std::fs::read(&path).map_err(|e| LedgerLoadError::Io {
                path: path.clone(),
                source: e.to_string(),
            })?;
            let env =
                FormEnvelope::from_json_bytes(&bytes).map_err(|e| LedgerLoadError::ParseError {
                    path: path.clone(),
                    source: e,
                })?;
            ledger.insert(env);
        }
        Ok(ledger)
    }
}

fn is_envelope_path(path: &Path) -> bool {
    // Match either `*.envelope.json` or any plain `*.json` so that
    // single-file demos can use either convention. The `*.envelope`
    // suffix is the recommended convention but we do not enforce it.
    // `file_type`-based file/directory filtering is done upstream by
    // `load_from_dir`; this function is a pure name test.
    let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
        return false;
    };
    name.ends_with(".envelope.json") || name.ends_with(".json")
}

/// All the ways `Ledger::load_from_dir` can fail. Surfaced separately
/// from `VerifyError` because a ledger problem is upstream of any
/// individual envelope's verification.
///
/// Hand-rolled `Display` rather than `thiserror` because the `path`
/// field is a `PathBuf` (no `Display` impl) and needs `.display()`.
#[derive(Debug)]
pub enum LedgerLoadError {
    Io {
        path: PathBuf,
        source: String,
    },
    ParseError {
        path: PathBuf,
        source: EnvelopeParseError,
    },
}

impl std::fmt::Display for LedgerLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LedgerLoadError::Io { path, source } => {
                write!(f, "ledger I/O error at {}: {}", path.display(), source)
            }
            LedgerLoadError::ParseError { path, source } => {
                write!(f, "ledger parse error at {}: {}", path.display(), source)
            }
        }
    }
}

impl std::error::Error for LedgerLoadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            LedgerLoadError::ParseError { source, .. } => Some(source),
            _ => None,
        }
    }
}

/// One distinct reason the verifier may refuse an envelope.
///
/// Each variant maps to one line of the structured `explain` output.
/// Keep variants flat (no nested errors) so callers can match directly.
///
/// A hand-rolled `Display` impl is used here (rather than `thiserror`)
/// because the `HashMismatch` variant invokes the `short` helper on its
/// fields to truncate long hex hashes, and that is awkward to express
/// in the `#[error(...)]` format-args grammar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VerifyError {
    /// The `hash` field disagrees with the recomputed canonical hash.
    HashMismatch { expected: String, actual: String },
    /// The form claims a parent that is not present in the ledger.
    MissingParent { parent: String },
    /// The form has empty parents but its `rule` is not `"genesis"`.
    /// A form with no derivation predecessor and no genesis claim is
    /// an orphan.
    OrphanForm { rule: String },
    /// The form's `proof_status` is `Invalid` — execution is denied
    /// at the gate, not even restricted.
    InvalidProofStatus,
    /// The payload requires a capability that is not declared.
    /// Reported once per missing op so the user can fix all of them.
    UndeclaredCapability {
        op_index: usize,
        op_name: &'static str,
        required: &'static str,
        declared: Vec<String>,
    },
    /// The form has open obligations and is marked `Verified`. This
    /// is a soft inconsistency the verifier surfaces but does not
    /// itself reject on (the runner will further restrict if needed);
    /// treated as a warning class. Listed here for completeness.
    UnresolvedObligations { obligations: Vec<String> },
}

impl std::fmt::Display for VerifyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VerifyError::HashMismatch { expected, actual } => write!(
                f,
                "hash mismatch (envelope claims {}, canonical is {})",
                short(expected),
                short(actual)
            ),
            VerifyError::MissingParent { parent } => {
                write!(f, "missing parent: {}", parent)
            }
            VerifyError::OrphanForm { rule } => write!(
                f,
                "orphan form: parents are empty but rule is {:?}, not \"genesis\"",
                rule
            ),
            VerifyError::InvalidProofStatus => {
                write!(f, "proof_status is invalid — execution denied at the gate")
            }
            VerifyError::UndeclaredCapability {
                op_index,
                op_name,
                required,
                declared,
            } => write!(
                f,
                "undeclared capability: op[{}] ({}) requires {:?}, declared {:?}",
                op_index, op_name, required, declared
            ),
            VerifyError::UnresolvedObligations { obligations } => write!(
                f,
                "unresolved obligations on a verified form: {:?}",
                obligations
            ),
        }
    }
}

impl std::error::Error for VerifyError {}

/// What the verifier returns when the envelope is admissible.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifyOutcome {
    /// The proof status carried through to the runner.
    pub proof_status: ProofStatus,
    /// Non-fatal observations — currently just unresolved obligations
    /// when proof_status is `Verified`. The runner does not gate on
    /// these but `explain` reports them.
    pub warnings: Vec<VerifyError>,
}

/// Run the verifier against `env` using `ledger` for parent lookups.
///
/// On success returns the proof status and any warnings. On the first
/// hard failure (hash, parent, orphan, invalid status, undeclared
/// capability) returns `Err(VerifyError)`.
///
/// Capability checks scan all ops and report on the *first* missing
/// declaration; subsequent missing caps are not reported in the same
/// run. The `explain` command can re-run after a fix to surface the
/// next one. (We could collect all of them, but a single failure at a
/// time keeps the gate's contract sharp: pass means pass.)
pub fn verify(env: &FormEnvelope, ledger: &Ledger) -> Result<VerifyOutcome, VerifyError> {
    // 1. Hash must match canonical recomputation.
    let canonical = env.compute_canonical_hash();
    if env.hash != canonical {
        return Err(VerifyError::HashMismatch {
            expected: env.hash.clone(),
            actual: canonical,
        });
    }

    // 2. Derivation: either genesis or every parent must be in the ledger.
    if env.parents.is_empty() {
        if !env.is_genesis() {
            return Err(VerifyError::OrphanForm {
                rule: env.rule.clone(),
            });
        }
    } else {
        for parent in &env.parents {
            if !ledger.contains(parent) {
                return Err(VerifyError::MissingParent {
                    parent: parent.clone(),
                });
            }
        }
    }

    // 3. Proof status: Invalid is a hard gate failure. Verified and
    //    Deferred both *admit* the form; the runner picks the mode.
    if env.proof_status == ProofStatus::Invalid {
        return Err(VerifyError::InvalidProofStatus);
    }

    // 4. Capability declaration: every op's required capability must
    //    be in env.capabilities. This is independent of execution
    //    mode — the gate refuses an envelope that is structurally
    //    inconsistent (asks to run an op it has not declared).
    //    Build a set once rather than linear-scanning per op.
    let declared: HashSet<&str> = env.capabilities.iter().map(String::as_str).collect();
    for (i, op) in env.payload.ops.iter().enumerate() {
        if let Some(required) = op.required_capability() {
            if !declared.contains(required) {
                return Err(VerifyError::UndeclaredCapability {
                    op_index: i,
                    op_name: op.name(),
                    required,
                    declared: env.capabilities.clone(),
                });
            }
        }
    }

    // 5. Soft warning: open obligations on a Verified form. Not a
    //    rejection — the runner is allowed to proceed — but worth
    //    surfacing in `explain`.
    let mut warnings = Vec::new();
    if env.proof_status == ProofStatus::Verified && !env.open_obligations.is_empty() {
        warnings.push(VerifyError::UnresolvedObligations {
            obligations: env.open_obligations.clone(),
        });
    }

    Ok(VerifyOutcome {
        proof_status: env.proof_status,
        warnings,
    })
}

fn short(hash: &str) -> String {
    if hash.len() <= 12 {
        hash.to_string()
    } else {
        format!("{}…", &hash[..12])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::envelope::{Op, Payload, GENESIS_RULE};

    fn genesis(form_id: &str, ops: Vec<Op>, caps: Vec<String>) -> FormEnvelope {
        FormEnvelope {
            form_id: form_id.into(),
            hash: String::new(),
            parents: vec![],
            rule: GENESIS_RULE.into(),
            proof_status: ProofStatus::Verified,
            open_obligations: vec![],
            capabilities: caps,
            payload: Payload { ops },
        }
        .with_canonical_hash()
    }

    #[test]
    fn verify_passes_for_clean_genesis() {
        let env = genesis("g", vec![Op::Emit { text: "hi".into() }], vec![]);
        let out = verify(&env, &Ledger::new()).unwrap();
        assert_eq!(out.proof_status, ProofStatus::Verified);
        assert!(out.warnings.is_empty());
    }

    #[test]
    fn verify_rejects_hash_mismatch() {
        let mut env = genesis("g", vec![Op::Emit { text: "hi".into() }], vec![]);
        env.hash = "0".repeat(64);
        let err = verify(&env, &Ledger::new()).unwrap_err();
        assert!(matches!(err, VerifyError::HashMismatch { .. }));
    }

    #[test]
    fn verify_rejects_orphan() {
        let mut env = genesis("g", vec![Op::Emit { text: "hi".into() }], vec![]);
        env.rule = "not-genesis".into();
        let env = env.with_canonical_hash();
        let err = verify(&env, &Ledger::new()).unwrap_err();
        assert!(matches!(err, VerifyError::OrphanForm { .. }));
    }

    #[test]
    fn verify_rejects_missing_parent() {
        let mut env = genesis("g", vec![Op::Emit { text: "hi".into() }], vec![]);
        env.rule = "step".into();
        env.parents = vec!["unknown-parent".into()];
        let env = env.with_canonical_hash();
        let err = verify(&env, &Ledger::new()).unwrap_err();
        assert!(matches!(err, VerifyError::MissingParent { .. }));
    }

    #[test]
    fn verify_admits_form_when_parent_is_in_ledger() {
        let parent = genesis("g", vec![Op::Emit { text: "hi".into() }], vec![]);
        let mut ledger = Ledger::new();
        ledger.insert(parent);

        let mut child = genesis("g/step", vec![Op::Emit { text: "x".into() }], vec![]);
        child.rule = "step".into();
        child.parents = vec!["g".into()];
        let child = child.with_canonical_hash();

        verify(&child, &ledger).unwrap();
    }

    #[test]
    fn verify_rejects_invalid_proof_status() {
        let mut env = genesis("g", vec![Op::Emit { text: "hi".into() }], vec![]);
        env.proof_status = ProofStatus::Invalid;
        let env = env.with_canonical_hash();
        let err = verify(&env, &Ledger::new()).unwrap_err();
        assert!(matches!(err, VerifyError::InvalidProofStatus));
    }

    #[test]
    fn verify_rejects_undeclared_capability() {
        let env = genesis(
            "g",
            vec![Op::Write {
                path: "/tmp/x".into(),
                content: "y".into(),
            }],
            vec![], // no caps declared
        );
        let err = verify(&env, &Ledger::new()).unwrap_err();
        match err {
            VerifyError::UndeclaredCapability {
                op_index, required, ..
            } => {
                assert_eq!(op_index, 0);
                assert_eq!(required, "io.fs");
            }
            other => panic!("expected UndeclaredCapability, got {:?}", other),
        }
    }

    #[test]
    fn verify_admits_when_capability_declared() {
        let env = genesis(
            "g",
            vec![Op::Write {
                path: "/tmp/x".into(),
                content: "y".into(),
            }],
            vec!["io.fs".into()],
        );
        verify(&env, &Ledger::new()).unwrap();
    }

    #[test]
    fn verify_warns_on_open_obligations_for_verified_form() {
        let mut env = genesis("g", vec![Op::Emit { text: "hi".into() }], vec![]);
        env.open_obligations = vec!["S-08:#3".into()];
        let env = env.with_canonical_hash();
        let out = verify(&env, &Ledger::new()).unwrap();
        assert_eq!(out.warnings.len(), 1);
        assert!(matches!(
            out.warnings[0],
            VerifyError::UnresolvedObligations { .. }
        ));
    }
}
