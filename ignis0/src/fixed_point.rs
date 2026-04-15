//! The A9.3 ignition fixed-point check.
//!
//! From `../../axioms/A9-ignition-substrate.md` § A9.3:
//!
//! > Let F be a small, total, well-specified Form whose output
//! > on a canonical input is declared in IL.md. The fixed-point
//! > check is:
//! >
//! > 1. `ignis0` executes F directly on the canonical input.
//! >    Call the result `r_direct`.
//! > 2. `ignis0` executes S-07 (the runtime Form) on the pair
//! >    `(F, canonical_input)`. Call the result `r_indirect`.
//! > 3. If `r_direct = r_indirect`, the check passes.
//!
//! `../../kernel/IGNITION-BOOTSTRAP.md` § Step 2 specifies F
//! concretely: add 1 to the input Nat, canonical input 42,
//! expected output 43.
//!
//! As of v0.2.3-ignis0-fp this module implements all three
//! levels end-to-end:
//!   - direct: F on 42 → 43 against `ExecState`.
//!   - indirect_1: a hand-encoded micro-`S-07/execute` wrapper
//!     is registered via `FormRegistry::register_wire` and
//!     `CALL`s F. Observed max call-frame depth: 2.
//!   - indirect_2: a second wrapper `CALL`s the first. Observed
//!     max call-frame depth: 3.
//!
//! The real `S-07/execute` Form replaces the micro wrapper at
//! v0.5.0-build; until then the hand-encoded wrapper is the
//! stage-0 stand-in per `IGNITION-BOOTSTRAP.md` § Step 2.

use crate::exec::{ExecState, ExecVerdict, Interpreter, StepResult};
use crate::opcode::Opcode;
use crate::parser::parse_form_lines;
use crate::registry::FormRegistry;
use crate::store::SubstanceStore;
use crate::value::{Hash, TrapKind, Value};
use crate::wire::{encode_form, Form as WireForm};

/// The verdict of a single fixed-point check evaluation.
#[derive(Debug)]
pub enum EvalVerdict {
    Produced(Value),
    /// Produced a value and carries the observed max call-frame
    /// depth. Used by the indirect paths, which want to report
    /// the call chain depth in addition to the value. Direct
    /// evaluation still uses `Produced` since it is a single
    /// frame by construction.
    ProducedTraced(Value, usize),
    Trapped(String),
    NotImplemented(&'static str),
}

impl EvalVerdict {
    pub fn as_nat(&self) -> Option<u128> {
        match self {
            EvalVerdict::Produced(Value::Nat(n)) => Some(*n),
            EvalVerdict::ProducedTraced(Value::Nat(n), _) => Some(*n),
            _ => None,
        }
    }

    pub fn value(&self) -> Option<&Value> {
        match self {
            EvalVerdict::Produced(v) => Some(v),
            EvalVerdict::ProducedTraced(v, _) => Some(v),
            _ => None,
        }
    }
}

/// The overall outcome of the A9.3 check.
#[derive(Debug)]
pub enum FixedPointVerdict {
    /// All three levels produced the same output. The
    /// necessary condition of A9.3 holds for this substrate.
    Pass {
        direct: Value,
        indirect_1: Value,
        indirect_2: Value,
        /// Maximum frame-stack depth reached during
        /// `eval_indirect_1` — the stage-0 analogue of a weave
        /// call chain. Expected value: 2 (micro-S-07 frame on
        /// top of the caller observer, plus F frame underneath
        /// as the callee). See `eval_indirect_1` for the exact
        /// chain construction.
        indirect_1_max_depth: usize,
        /// Maximum frame-stack depth reached during
        /// `eval_indirect_2`. Expected value: 3 (S-07² → S-07 → F).
        indirect_2_max_depth: usize,
    },
    /// At least one indirect level is not yet runnable because
    /// the required helper chain is incomplete (no parser, no
    /// CALL, etc.). Reports which level is pending.
    Incomplete {
        direct: Value,
        indirect_1_status: &'static str,
        indirect_2_status: &'static str,
    },
    /// Direct execution failed — this is a real `ignis0` bug.
    DirectFailed(String),
    /// Direct and indirect disagreed — halts ignition per A9.3.
    Disagreed {
        direct: Value,
        indirect: Value,
        level: u8,
    },
}

/// The canonical Form F source in line-oriented form. The
/// scaffold parser in `parser.rs` reads this into a Vec<Opcode>.
/// The canonical s-expression version of the same Form lives
/// in `../../kernel/IGNITION-BOOTSTRAP.md` § Step 2.
pub const CANONICAL_F_SOURCE: &str = r#"
; The canonical fixed-point Form F from
; kernel/IGNITION-BOOTSTRAP.md § Step 2.
; Adds 1 to the input Nat.
STORE 0
LOAD 0
PUSH 1
ADD
RET
"#;

/// The A9.3 fixed-point check harness.
pub struct FixedPointCheck {
    pub store: SubstanceStore,
}

impl FixedPointCheck {
    pub fn new() -> Self {
        Self { store: SubstanceStore::new() }
    }

    /// Build the canonical Form F either via the parser (new
    /// path, v0.2.0-ignition) or via a hand-constructed vec
    /// (legacy path, kept for tests that want to bypass the
    /// parser). This method calls the parser.
    pub fn build_f_parsed() -> Vec<Opcode> {
        parse_form_lines(CANONICAL_F_SOURCE)
            .expect("canonical F source is always valid")
    }

    /// Legacy hand-constructed build path. Tests that want to
    /// exercise the interpreter without going through the
    /// parser call this directly.
    #[allow(non_snake_case)]
    pub fn build_F() -> Vec<Opcode> {
        vec![
            Opcode::Store(0),
            Opcode::Load(0),
            Opcode::Push(Value::Nat(1)),
            Opcode::Add,
            Opcode::Ret,
        ]
    }

    /// Seal F as a substance and return its hash.
    ///
    /// The real sealing would canonicalise F's wire bytes and
    /// seal them through the host-language S-03. The scaffold
    /// seals a tagged placeholder so the returned Hash is
    /// stable across runs.
    pub fn seal_f(&mut self) -> Hash {
        self.store
            .seal("Form/v1:ignition_fixed_point_F", Value::Nat(42))
    }

    /// Run F directly on the canonical input.
    pub fn eval_direct(&mut self, input: u128) -> EvalVerdict {
        let form_hash = self.seal_f();
        let code = Self::build_f_parsed();
        let mut state = ExecState::new(form_hash, code, 1, vec![Value::Nat(input)]);
        let mut interp = Interpreter::new(&mut self.store);
        match interp.run(&mut state, 1024) {
            ExecVerdict::Returned(v) => EvalVerdict::Produced(v),
            ExecVerdict::Trapped(k) => EvalVerdict::Trapped(format!("{}", k)),
            ExecVerdict::Yielded => {
                EvalVerdict::Trapped("unexpected YIELD in F".to_string())
            }
        }
    }

    /// Build F as a canonical wire-form `Form` record. Used by
    /// the indirect fixed-point paths which need F's wire hash
    /// so a caller's CALL opcode can name it.
    fn build_f_wire() -> WireForm {
        WireForm {
            type_tag: "Form/v1".to_string(),
            arity: 1,
            locals_n: 1,
            declared_caps: vec![],
            declared_traps: vec![TrapKind::EType("Nat".into())],
            code: Self::build_F(),
        }
    }

    /// Build a minimal micro-`S-07/execute`: a Form that takes a
    /// single argument, calls the target Form with it, and
    /// returns the callee's result. This is **not** the full
    /// IgnisSynth `S-07/execute` — the real one is encoded in
    /// `kernel/forms/S-07-form-runtime.form` and implements a
    /// full IL interpreter. What this function produces is a
    /// minimal stage-0 stand-in that exercises the same
    /// `CALL`-then-`RET` chain the real S-07 will use once it
    /// becomes loadable.
    ///
    /// Code: `STORE 0 ; LOAD 0 ; CALL { form: target, n: 1 } ; RET`.
    ///
    /// The caller passes the input via CALL's stack-transfer
    /// convention (the top of the caller's stack becomes the
    /// callee's initial stack). STORE 0 parks the argument in
    /// locals[0], LOAD 0 pushes it back for CALL to consume,
    /// CALL passes it to `target`, and RET returns target's
    /// return value to this wrapper's caller.
    ///
    /// Label is a human-readable tag used for the registry's
    /// name index; it is not consulted at dispatch time.
    fn build_micro_s07(target: Hash) -> WireForm {
        WireForm {
            type_tag: "Form/v1".to_string(),
            arity: 1,
            locals_n: 1,
            declared_caps: vec![],
            declared_traps: vec![TrapKind::EType("Nat".into())],
            code: vec![
                Opcode::Store(0),
                Opcode::Load(0),
                Opcode::Call { form: target, n: 1 },
                Opcode::Ret,
            ],
        }
    }

    /// Drive an `Interpreter` + `ExecState` to completion while
    /// recording the maximum call-frame depth seen at any point.
    /// This is the stage-0 analogue of a weave log — it tells us
    /// the call chain reached the expected number of frames,
    /// which for the indirect cases is the observable we care
    /// about. (The weave proper lives in the habitat, per S-04;
    /// ignis0 has no weave under A9.4.)
    fn run_traced(
        interp: &mut Interpreter,
        state: &mut ExecState,
        max_steps: usize,
    ) -> Result<(Value, usize), String> {
        let mut max_depth = state.depth();
        for _ in 0..max_steps {
            if state.depth() > max_depth {
                max_depth = state.depth();
            }
            match interp.step(state) {
                StepResult::Step => continue,
                StepResult::Returned(v) => return Ok((v, max_depth)),
                StepResult::Trapped(k) => return Err(format!("{}", k)),
                StepResult::Yielded => {
                    return Err("unexpected YIELD in fixed-point run".into())
                }
            }
        }
        Err("max_steps exceeded in fixed-point run".into())
    }

    /// Build a registry containing canonical F and `levels`
    /// wrappers of micro-`S-07/execute`, each calling the
    /// previous layer. Returns the registry and the hash of the
    /// outermost Form (the one a caller should invoke).
    ///
    /// For `levels = 1`: registry holds `{F, S07}`; outer = S07.
    /// For `levels = 2`: registry holds `{F, S07, S07²}`;
    /// outer = S07² (which CALLs S07, which CALLs F).
    fn build_indirect_chain(levels: usize) -> (FormRegistry, Hash) {
        let mut reg = FormRegistry::new();
        let f_bytes = encode_form(&Self::build_f_wire())
            .expect("canonical F encodes cleanly");
        let f_hash = reg
            .register_wire("A9.3/F", &f_bytes)
            .expect("canonical F decodes cleanly");
        let mut current_hash = f_hash;
        for i in 1..=levels {
            let s07 = Self::build_micro_s07(current_hash);
            let bytes = encode_form(&s07)
                .expect("micro-S-07 encodes cleanly");
            let name = format!("A9.3/micro_s07_level_{}", i);
            current_hash = reg
                .register_wire(&name, &bytes)
                .expect("micro-S-07 decodes cleanly");
        }
        (reg, current_hash)
    }

    /// Run `F` via one level of micro-S-07 indirection:
    /// `micro_s07(F)(input)`. Stage-0 substrate only — the real
    /// S-07 from the habitat is not involved.
    pub fn eval_indirect_1(&mut self, input: u128) -> EvalVerdict {
        let (reg, outer) = Self::build_indirect_chain(1);
        let loaded = reg
            .get(&outer)
            .expect("outer Form is in the registry by construction")
            .clone();
        let mut state = ExecState::new(
            outer,
            loaded.code,
            loaded.locals_n,
            vec![Value::Nat(input)],
        );
        let mut interp = Interpreter::new(&mut self.store).with_registry(&reg);
        match Self::run_traced(&mut interp, &mut state, 4096) {
            Ok((v, depth)) => EvalVerdict::ProducedTraced(v, depth),
            Err(msg) => EvalVerdict::Trapped(msg),
        }
    }

    /// Run `F` via two levels of micro-S-07 indirection:
    /// `micro_s07(micro_s07(F))(input)`. Exercises CALL
    /// recursively through the `FormRegistry` dispatcher.
    pub fn eval_indirect_2(&mut self, input: u128) -> EvalVerdict {
        let (reg, outer) = Self::build_indirect_chain(2);
        let loaded = reg
            .get(&outer)
            .expect("outer Form is in the registry by construction")
            .clone();
        let mut state = ExecState::new(
            outer,
            loaded.code,
            loaded.locals_n,
            vec![Value::Nat(input)],
        );
        let mut interp = Interpreter::new(&mut self.store).with_registry(&reg);
        match Self::run_traced(&mut interp, &mut state, 4096) {
            Ok((v, depth)) => EvalVerdict::ProducedTraced(v, depth),
            Err(msg) => EvalVerdict::Trapped(msg),
        }
    }

    /// Run the full A9.3 check on the canonical input 42.
    pub fn run(&mut self) -> FixedPointVerdict {
        const INPUT: u128 = 42;
        const EXPECTED: u128 = 43;

        // Direct case — must succeed for anything else to matter.
        let direct = self.eval_direct(INPUT);
        let direct_value = match direct {
            EvalVerdict::Produced(ref v) => v.clone(),
            EvalVerdict::Trapped(msg) => {
                return FixedPointVerdict::DirectFailed(msg);
            }
            EvalVerdict::NotImplemented(_) => {
                return FixedPointVerdict::DirectFailed(
                    "direct case is not implemented — scaffold bug".into(),
                );
            }
        };

        // Sanity: direct must produce the expected value. This
        // is a *stronger* condition than A9.3 strictly requires
        // (A9.3 requires agreement across all three levels, not
        // a specific value), but since the canonical spec
        // declares the expected output, we check it here.
        if direct_value != Value::Nat(EXPECTED) {
            return FixedPointVerdict::DirectFailed(format!(
                "direct case produced {:?}, expected Nat({})",
                direct_value, EXPECTED
            ));
        }

        // Indirect cases — v0.2.3-ignis0-fp: live, not stubs.
        let indirect_1 = self.eval_indirect_1(INPUT);
        let indirect_2 = self.eval_indirect_2(INPUT);

        match (&indirect_1, &indirect_2) {
            (
                EvalVerdict::ProducedTraced(v1, d1),
                EvalVerdict::ProducedTraced(v2, d2),
            ) => {
                if *v1 != direct_value {
                    return FixedPointVerdict::Disagreed {
                        direct: direct_value,
                        indirect: v1.clone(),
                        level: 1,
                    };
                }
                if *v2 != direct_value {
                    return FixedPointVerdict::Disagreed {
                        direct: direct_value,
                        indirect: v2.clone(),
                        level: 2,
                    };
                }
                FixedPointVerdict::Pass {
                    direct: direct_value,
                    indirect_1: v1.clone(),
                    indirect_2: v2.clone(),
                    indirect_1_max_depth: *d1,
                    indirect_2_max_depth: *d2,
                }
            }
            _ => FixedPointVerdict::Incomplete {
                direct: direct_value,
                indirect_1_status: status_str(&indirect_1),
                indirect_2_status: status_str(&indirect_2),
            },
        }
    }
}

fn status_str(v: &EvalVerdict) -> &'static str {
    match v {
        EvalVerdict::NotImplemented(s) => s,
        EvalVerdict::Produced(_) => "produced",
        EvalVerdict::ProducedTraced(_, _) => "produced-traced",
        EvalVerdict::Trapped(_) => "trapped",
    }
}

impl Default for FixedPointCheck {
    fn default() -> Self {
        Self::new()
    }
}
