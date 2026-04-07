//! Value types: the things that live on the stack, in locals,
//! and in the substance store.

use std::fmt;

/// A 32-byte BLAKE3 substance hash. Content-addressed identity
/// per S-03 obligation 1 (hash determinism).
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
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

/// Alias used in public API for brevity.
pub type Hash = SubstanceHash;

/// Values the interpreter pushes onto the stack and stores in
/// locals. The closed set of shapes the IL can manipulate.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Value {
    Unit,
    Bool(bool),
    Nat(u64),
    Hash(Hash),
    Bytes(Vec<u8>),
    Pair(Box<Value>, Box<Value>),
    Vec(Vec<Value>),
}

impl Value {
    /// Try to extract a `Nat`. Returns `Err(TrapKind::ETYPE)` on
    /// mismatch — the rule for every arithmetic opcode.
    pub fn as_nat(&self) -> Result<u64, TrapKind> {
        match self {
            Value::Nat(n) => Ok(*n),
            _ => Err(TrapKind::ETYPE),
        }
    }

    /// Try to extract a `Bool`.
    pub fn as_bool(&self) -> Result<bool, TrapKind> {
        match self {
            Value::Bool(b) => Ok(*b),
            _ => Err(TrapKind::ETYPE),
        }
    }
}

/// Closed enumeration of trap kinds from
/// `../../kernel/IL.md` § Trap kinds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrapKind {
    EBadLocal,
    EType,
    EUnderflow,
    EUnheld,
    ENotHeld,
    EStale,
    EOverBudget,
    EAssert,
    EUnauthorised,
    EIgnited,
    EReplayDiverged,
    /// Not in IL.md; used by this scaffold to flag opcodes
    /// whose implementation is not yet written. A real `ignis0`
    /// would not have this variant — it would implement all
    /// 34 opcodes.
    NotImplemented,
}

// Shorter aliases matching the spec's naming convention.
#[allow(non_upper_case_globals)]
impl TrapKind {
    pub const ETYPE: TrapKind = TrapKind::EType;
    pub const EBADLOCAL: TrapKind = TrapKind::EBadLocal;
    pub const EUNDERFLOW: TrapKind = TrapKind::EUnderflow;
    pub const EUNHELD: TrapKind = TrapKind::EUnheld;
    pub const EASSERT: TrapKind = TrapKind::EAssert;
}
