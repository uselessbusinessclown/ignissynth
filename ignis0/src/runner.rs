//! Envelope runner — execution under a derivation-derived mode.
//!
//! The runner takes a `FormEnvelope` and a `VerifyOutcome` from
//! `verify::verify` and produces an `EnvelopeRunResult` describing
//! what happened. Three modes apply, chosen from the envelope's
//! proof status:
//!
//! - `Full` — all declared ops execute, subject to capability checks
//!   already performed by the verifier.
//! - `Restricted` — only observable-only ops execute. Anything that
//!   would have a side effect outside the runner is reported as
//!   `OpDecision::SkippedRestricted` and not executed.
//! - `Denied` — the runner refuses to execute anything. Returned
//!   when proof status is `Invalid` (which the verifier already
//!   refuses to admit) or when the caller forces denied mode.
//!
//! Crucially, the runner does not re-check the gates the verifier
//! already covered. It assumes its input has been verified.

use crate::envelope::{FormEnvelope, Op, ProofStatus};
use crate::verify::VerifyOutcome;

/// The execution policy applied to one run of an envelope.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnvelopeMode {
    /// All ops permitted (capabilities already checked at verify time).
    Full,
    /// Only `Op::is_observable_only` ops permitted; others reported
    /// as skipped without running.
    Restricted,
    /// No ops permitted. The runner returns immediately.
    Denied,
}

impl EnvelopeMode {
    /// Choose the mode for an envelope from its proof status. This is
    /// the canonical mapping; callers can override (e.g. force
    /// restricted mode for a verified form during a dry run).
    pub fn for_status(status: ProofStatus) -> Self {
        match status {
            ProofStatus::Verified => EnvelopeMode::Full,
            ProofStatus::Deferred => EnvelopeMode::Restricted,
            ProofStatus::Invalid => EnvelopeMode::Denied,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            EnvelopeMode::Full => "full",
            EnvelopeMode::Restricted => "restricted",
            EnvelopeMode::Denied => "denied",
        }
    }
}

/// Per-op outcome from the runner. One of these is recorded for every
/// op in the payload, in declaration order, regardless of mode.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OpDecision {
    /// The op ran. The string is the human-readable effect (for
    /// `Emit`, the emitted text; for `Write` and `Infer`, a stub
    /// description — these capabilities are simulated by the runner
    /// rather than reaching the network or filesystem, since the
    /// envelope demo is not the right place to ship side effects).
    Executed { effect: String },
    /// The op was skipped because the runner is in `Restricted` mode
    /// and the op is not observable-only.
    SkippedRestricted { op_name: &'static str },
    /// The runner is in `Denied` mode and the op was not considered.
    Denied,
}

/// Top-level result of running an envelope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnvelopeRunResult {
    pub mode: EnvelopeMode,
    pub decisions: Vec<OpDecision>,
}

impl EnvelopeRunResult {
    pub fn executed_count(&self) -> usize {
        self.decisions
            .iter()
            .filter(|d| matches!(d, OpDecision::Executed { .. }))
            .count()
    }

    pub fn skipped_count(&self) -> usize {
        self.decisions
            .iter()
            .filter(|d| matches!(d, OpDecision::SkippedRestricted { .. }))
            .count()
    }

    pub fn denied_count(&self) -> usize {
        self.decisions
            .iter()
            .filter(|d| matches!(d, OpDecision::Denied))
            .count()
    }
}

/// Run an envelope under the mode dictated by `outcome.proof_status`.
///
/// Side effects are intentionally simulated: `Op::Write` and
/// `Op::Infer` produce a descriptive `Executed { effect: "..." }`
/// without touching the filesystem or network. The point of the demo
/// is to prove the gating works; introducing real side effects here
/// would invite the runner to *be* the substrate, which it is not.
pub fn run_envelope(env: &FormEnvelope, outcome: &VerifyOutcome) -> EnvelopeRunResult {
    let mode = EnvelopeMode::for_status(outcome.proof_status);
    run_envelope_with_mode(env, mode)
}

/// Run an envelope under an explicitly chosen mode. Useful when the
/// caller wants to dry-run a verified form under restricted mode.
pub fn run_envelope_with_mode(env: &FormEnvelope, mode: EnvelopeMode) -> EnvelopeRunResult {
    let decisions = match mode {
        EnvelopeMode::Denied => env
            .payload
            .ops
            .iter()
            .map(|_| OpDecision::Denied)
            .collect(),
        EnvelopeMode::Restricted | EnvelopeMode::Full => env
            .payload
            .ops
            .iter()
            .map(|op| decide_one(op, mode))
            .collect(),
    };
    EnvelopeRunResult { mode, decisions }
}

fn decide_one(op: &Op, mode: EnvelopeMode) -> OpDecision {
    if mode == EnvelopeMode::Restricted && !op.is_observable_only() {
        return OpDecision::SkippedRestricted {
            op_name: op.name(),
        };
    }
    OpDecision::Executed {
        effect: simulate_effect(op),
    }
}

fn simulate_effect(op: &Op) -> String {
    match op {
        Op::Emit { text } => format!("emit: {}", text),
        Op::Write { path, content } => format!(
            "write (simulated): would write {} bytes to {}",
            content.len(),
            path
        ),
        Op::Infer { prompt } => format!(
            "infer (simulated): would submit {}-char prompt to infer.remote",
            prompt.len()
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::envelope::{Op, Payload, GENESIS_RULE};
    use crate::verify::Ledger;

    fn make_env(status: ProofStatus, ops: Vec<Op>, caps: Vec<String>) -> FormEnvelope {
        FormEnvelope {
            form_id: "g".into(),
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

    fn verified_outcome(env: &FormEnvelope) -> VerifyOutcome {
        crate::verify::verify(env, &Ledger::new()).expect("test fixture must verify")
    }

    #[test]
    fn mode_mapping_is_canonical() {
        assert_eq!(EnvelopeMode::for_status(ProofStatus::Verified), EnvelopeMode::Full);
        assert_eq!(
            EnvelopeMode::for_status(ProofStatus::Deferred),
            EnvelopeMode::Restricted
        );
        assert_eq!(EnvelopeMode::for_status(ProofStatus::Invalid), EnvelopeMode::Denied);
    }

    #[test]
    fn full_mode_executes_all_ops() {
        let env = make_env(
            ProofStatus::Verified,
            vec![
                Op::Emit { text: "hi".into() },
                Op::Write {
                    path: "/tmp/x".into(),
                    content: "y".into(),
                },
            ],
            vec!["io.fs".into()],
        );
        let outcome = verified_outcome(&env);
        let result = run_envelope(&env, &outcome);
        assert_eq!(result.mode, EnvelopeMode::Full);
        assert_eq!(result.executed_count(), 2);
        assert_eq!(result.skipped_count(), 0);
    }

    #[test]
    fn restricted_mode_skips_side_effecting_ops() {
        let env = make_env(
            ProofStatus::Deferred,
            vec![
                Op::Emit { text: "hi".into() },
                Op::Write {
                    path: "/tmp/x".into(),
                    content: "y".into(),
                },
                Op::Infer { prompt: "p".into() },
            ],
            vec!["io.fs".into(), "infer.remote".into()],
        );
        let outcome = verified_outcome(&env);
        let result = run_envelope(&env, &outcome);
        assert_eq!(result.mode, EnvelopeMode::Restricted);
        assert_eq!(result.executed_count(), 1, "only emit may run in restricted mode");
        assert_eq!(result.skipped_count(), 2);
        // The single executed op must be the emit.
        assert!(matches!(
            &result.decisions[0],
            OpDecision::Executed { effect } if effect == "emit: hi"
        ));
    }

    #[test]
    fn denied_mode_runs_nothing() {
        let env = make_env(
            ProofStatus::Verified,
            vec![Op::Emit { text: "hi".into() }],
            vec![],
        );
        let result = run_envelope_with_mode(&env, EnvelopeMode::Denied);
        assert_eq!(result.mode, EnvelopeMode::Denied);
        assert_eq!(result.executed_count(), 0);
        assert_eq!(result.denied_count(), 1);
    }

    #[test]
    fn explicit_mode_overrides_status_mapping() {
        // A Verified form can be dry-run under Restricted mode.
        let env = make_env(
            ProofStatus::Verified,
            vec![Op::Write {
                path: "/tmp/x".into(),
                content: "y".into(),
            }],
            vec!["io.fs".into()],
        );
        let result = run_envelope_with_mode(&env, EnvelopeMode::Restricted);
        assert_eq!(result.mode, EnvelopeMode::Restricted);
        assert_eq!(result.skipped_count(), 1);
    }
}
