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
//! This scaffold implements the direct case end-to-end and
//! stubs the indirect cases honestly. Once `S-07/execute` is
//! loadable by this interpreter (which requires the IL parser
//! from `../../kernel/forms/helpers/parser.form` and a working
//! CALL opcode), the indirect cases can be enabled.

use crate::exec::{ExecState, ExecVerdict, Interpreter};
use crate::opcode::Opcode;
use crate::store::SubstanceStore;
use crate::value::{Hash, Value};

/// The verdict of a single fixed-point check evaluation.
#[derive(Debug)]
pub enum EvalVerdict {
    Produced(Value),
    Trapped(String),
    NotImplemented(&'static str),
}

impl EvalVerdict {
    pub fn as_nat(&self) -> Option<u64> {
        match self {
            EvalVerdict::Produced(Value::Nat(n)) => Some(*n),
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

/// The A9.3 fixed-point check harness.
pub struct FixedPointCheck {
    pub store: SubstanceStore,
}

impl FixedPointCheck {
    pub fn new() -> Self {
        Self { store: SubstanceStore::new() }
    }

    /// Build the canonical Form F from IGNITION-BOOTSTRAP.md:
    ///
    ///   (form
    ///     :name "ignition_fixed_point_F"
    ///     :arity 1
    ///     :locals-n 1
    ///     :code (
    ///       STORE 0
    ///       LOAD  0
    ///       PUSH  1
    ///       ADD
    ///       RET
    ///     ))
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
    pub fn seal_F(&mut self) -> Hash {
        // In the real spec, F is sealed as its canonical wire
        // bytes via the canonicaliser. For the scaffold, we
        // seal a tagged representation that is byte-deterministic
        // for the purposes of producing a stable hash. The actual
        // body is what the interpreter runs.
        self.store
            .seal("Form/v1:ignition_fixed_point_F", Value::Nat(42))
    }

    /// Run F directly on the canonical input.
    pub fn eval_direct(&mut self, input: u64) -> EvalVerdict {
        let form_hash = self.seal_F();
        let code = Self::build_F();
        let mut state = ExecState::new(form_hash, code, 1, vec![Value::Nat(input)]);
        let mut interp = Interpreter::new(&mut self.store);
        match interp.run(&mut state, 1024) {
            ExecVerdict::Returned(v) => EvalVerdict::Produced(v),
            ExecVerdict::Trapped(k) => EvalVerdict::Trapped(format!("{:?}", k)),
            ExecVerdict::Yielded => {
                EvalVerdict::Trapped("unexpected YIELD in F".to_string())
            }
        }
    }

    /// Run S-07 interpreting F. Stubbed — requires the IL parser
    /// and a working CALL opcode, which the scaffold does not
    /// yet provide.
    pub fn eval_indirect_1(&mut self, _input: u64) -> EvalVerdict {
        EvalVerdict::NotImplemented(
            "requires S-07/execute loaded via the IL parser + working CALL. \
             Tracked as ignis0 v0.2.0-scaffold milestone.",
        )
    }

    /// Run S-07 interpreting S-07 interpreting F. Stubbed for
    /// the same reason as indirect_1, plus one additional level
    /// of nesting.
    pub fn eval_indirect_2(&mut self, _input: u64) -> EvalVerdict {
        EvalVerdict::NotImplemented(
            "requires the indirect_1 case plus a CALL opcode that \
             recursively invokes S-07 on itself. Tracked as ignis0 \
             v0.2.0-scaffold milestone.",
        )
    }

    /// Run the full A9.3 check on the canonical input 42.
    pub fn run(&mut self) -> FixedPointVerdict {
        const INPUT: u64 = 42;
        const EXPECTED: u64 = 43;

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

        // Indirect cases — scaffold stubs.
        let indirect_1 = self.eval_indirect_1(INPUT);
        let indirect_2 = self.eval_indirect_2(INPUT);

        match (&indirect_1, &indirect_2) {
            (EvalVerdict::Produced(v1), EvalVerdict::Produced(v2)) => {
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
                }
            }
            _ => FixedPointVerdict::Incomplete {
                direct: direct_value,
                indirect_1_status: match &indirect_1 {
                    EvalVerdict::NotImplemented(s) => s,
                    EvalVerdict::Produced(_) => "produced",
                    EvalVerdict::Trapped(_) => "trapped",
                },
                indirect_2_status: match &indirect_2 {
                    EvalVerdict::NotImplemented(s) => s,
                    EvalVerdict::Produced(_) => "produced",
                    EvalVerdict::Trapped(_) => "trapped",
                },
            },
        }
    }
}

impl Default for FixedPointCheck {
    fn default() -> Self {
        Self::new()
    }
}
