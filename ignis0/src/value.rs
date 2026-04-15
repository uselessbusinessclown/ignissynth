//! Value types: the things that live on the stack, in locals,
//! and in the substance store.
//!
//! Design notes on decisions made in v0.2.0-ignition:
//!
//! - `Nat` widens to `u128`. IL.md does not bound Nats; the
//!   encoded primary Forms use small values (budget counters,
//!   offsets, indices) that fit in u64, but proofs care about
//!   arithmetic soundness in the limit. u128 gives enough
//!   margin for realistic seed workloads without pulling in
//!   a bignum dependency.
//!
//! - `Hash` stays a plain `[u8; 32]` newtype with a custom
//!   `Debug` that prints a short prefix. We add a `From<blake3::Hash>`
//!   impl so interop with the blake3 crate is ergonomic.
//!
//! - `TrapKind` uses `thiserror` for display messages. The
//!   variants match `kernel/IL.md § Trap kinds` one-for-one,
//!   plus one scaffold-only variant `NotImplemented` for
//!   opcodes whose body is pending. The variants match IL.md
//!   exactly — no renaming or removal — because a strict
//!   reading of the IL depends on them.
//!
//! - `Value` gains `Cont` and `Cell` variants. `Cont` will
//!   hold a sealed ExecState for YIELD; `Cell` is a substance
//!   reference distinct from a raw Hash so opcodes that
//!   expect one can trap ETYPE on the other.

use serde::{Deserialize, Serialize};
use std::fmt;

/// A 32-byte BLAKE3 substance hash. Content-addressed identity
/// per S-03 obligation 1 (hash determinism).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SubstanceHash(pub [u8; 32]);

impl SubstanceHash {
    /// The all-zeros hash. Used for `BOTTOM_HASH` and the
    /// initial tip of an empty weave.
    pub const BOTTOM: SubstanceHash = SubstanceHash([0u8; 32]);

    /// Compute the canonical hash of a byte sequence.
    pub fn of(bytes: &[u8]) -> Self {
        SubstanceHash(*blake3::hash(bytes).as_bytes())
    }

    /// Produce a short hex prefix for display.
    pub fn short(&self) -> String {
        let mut s = String::with_capacity(8);
        for b in &self.0[..4] {
            s.push_str(&format!("{:02x}", b));
        }
        s
    }
}

impl fmt::Debug for SubstanceHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Hash({}...)", self.short())
    }
}

impl From<blake3::Hash> for SubstanceHash {
    fn from(h: blake3::Hash) -> Self {
        SubstanceHash(*h.as_bytes())
    }
}

/// Alias used in public API for brevity.
pub type Hash = SubstanceHash;

/// Values the interpreter pushes onto the stack and stores in
/// locals. The closed set of shapes the IL can manipulate.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Value {
    Unit,
    Bool(bool),
    /// u128 for margin; IL.md does not specify a Nat width,
    /// and u128 is wide enough for every field in the encoded
    /// seed Forms. A proper ignis0 might later upgrade to a
    /// bignum if proofs require it.
    Nat(u128),
    Hash(Hash),
    Bytes(Vec<u8>),
    Pair(Box<Value>, Box<Value>),
    Vec(Vec<Value>),
    /// A substance reference — distinct from a raw `Hash` at
    /// the type level so opcodes that expect a Cell (e.g.,
    /// `READ` acting on a cell handle rather than an opaque
    /// hash) can trap `ETYPE` on the wrong variant.
    Cell(Hash),
    /// A sealed ExecState representing a yielded continuation.
    /// Used by YIELD and resume_continuation. For the v0.2.0
    /// scaffold this is a marker; the actual ExecState sits
    /// in the store and the Cont holds its hash.
    Cont(Hash),
}

impl Value {
    /// Try to extract a `Nat`. Returns `Err(TrapKind::EType)`
    /// on mismatch — the rule for every arithmetic opcode.
    pub fn as_nat(&self) -> Result<u128, TrapKind> {
        match self {
            Value::Nat(n) => Ok(*n),
            _ => Err(TrapKind::EType("expected Nat".to_string())),
        }
    }

    /// Try to extract a `Bool`.
    pub fn as_bool(&self) -> Result<bool, TrapKind> {
        match self {
            Value::Bool(b) => Ok(*b),
            _ => Err(TrapKind::EType("expected Bool".to_string())),
        }
    }

    /// Try to extract a `Hash`.
    pub fn as_hash(&self) -> Result<Hash, TrapKind> {
        match self {
            Value::Hash(h) => Ok(*h),
            Value::Cell(h) => Ok(*h),
            _ => Err(TrapKind::EType("expected Hash".to_string())),
        }
    }
}

/// Closed enumeration of trap kinds from
/// `../../kernel/IL.md` § Trap kinds.
///
/// Uses `thiserror` so the CLI can format verdicts cleanly.
/// Variants carry a short `String` for the operand / context,
/// which is useful for debugging scaffold bugs but would be
/// stripped (or made more structured) in a production ignis0.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum TrapKind {
    #[error("EBADLOCAL: {0}")]
    EBadLocal(String),
    #[error("ETYPE: {0}")]
    EType(String),
    #[error("EUNDERFLOW")]
    EUnderflow,
    #[error("EUNHELD: {0}")]
    EUnheld(String),
    #[error("ENOTHELD")]
    ENotHeld,
    #[error("ESTALE")]
    EStale,
    #[error("EOVERBUDGET")]
    EOverBudget,
    #[error("EASSERT")]
    EAssert,
    #[error("EUNAUTHORISED")]
    EUnauthorised,
    #[error("EIGNITED")]
    EIgnited,
    #[error("EREPLAYDIVERGED")]
    EReplayDiverged,
    /// Not in IL.md; used by this scaffold to flag opcodes
    /// whose implementation is not yet written. A real `ignis0`
    /// would not have this variant — it would implement all
    /// 35 opcodes.
    #[error("NotImplemented: {0}")]
    NotImplemented(String),
}

impl TrapKind {
    /// Construct an ETYPE with a canned message. Used by the
    /// interpreter when it doesn't have richer context.
    pub fn type_mismatch(ctx: &str) -> Self {
        TrapKind::EType(ctx.to_string())
    }

    /// Construct an EBADLOCAL with an index.
    pub fn bad_local(i: u32) -> Self {
        TrapKind::EBadLocal(format!("locals[{}]", i))
    }
}
