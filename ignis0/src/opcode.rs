//! The 35 IL opcodes from `../../kernel/IL.md` § Opcodes.
//!
//! Implementations of `step()` for each opcode live in
//! `exec.rs`. This file is the type-level enumeration.
//!
//! The 34→35 bump added `CALLI` (indirect call): it pops its
//! target hash from the stack, so `READSLOT + CALLI` composes
//! and slot-based dynamic dispatch becomes expressible. Direct
//! `CALL` (with an immediate hash) is retained for statically
//! known targets where the call graph should remain visible
//! without execution.

use crate::value::{Hash, TrapKind, Value};

/// One instruction in a Form's code vec.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Opcode {
    // --- Stack and locals (4) ---
    /// PUSH imm — push an immediate value onto the stack.
    Push(Value),
    /// POP — discard the top of the stack.
    Pop,
    /// LOAD i — push `locals[i]`. Traps `EBADLOCAL` on oob.
    Load(u32),
    /// STORE i — pop and write to `locals[i]`.
    Store(u32),

    // --- Arithmetic and comparison (4) ---
    /// ADD — pop two Nats, push their sum.
    Add,
    /// SUB — pop (b, a), push a-b; traps `EUNDERFLOW` if b > a.
    Sub,
    /// EQ — pop two values, push structural equality.
    Eq,
    /// LT — pop two Nats, push a<b.
    Lt,

    // --- Control flow (4) ---
    /// JMP off — pc += off (signed).
    Jmp(i32),
    /// JMPZ off — pop Bool; branch if false.
    Jmpz(i32),
    /// CALL h/n — call Form at immediate `h` with `n` stack
    /// args. Static target; visible in the call graph without
    /// execution.
    Call { form: Hash, n: u32 },
    /// CALLI n — call Form at stack-top hash with `n` stack
    /// args. Indirect target; composes with `READSLOT` to
    /// realise slot-based dynamic dispatch. Traps `ETYPE` if
    /// the top of the stack is not a `Hash`.
    CallI { n: u32 },
    /// RET — return top of stack to caller; emits one Invoked
    /// weave entry (in a real implementation).
    Ret,

    // --- Structure (4) ---
    /// MAKEPAIR — pop (b, a), push Pair{a, b}.
    MakePair,
    /// FST — pop Pair, push first.
    Fst,
    /// SND — pop Pair, push second.
    Snd,
    /// MAKEVEC n — pop n values, push a Vec of them.
    MakeVec(u32),

    // --- Substance (4) ---
    /// SEAL t — pop value, push `S-03.seal(t, value)`.
    Seal(String),
    /// READ — pop Hash, push underlying value; traps
    /// `EUNHELD` without a read cap.
    Read,
    /// PIN — pop Hash, `S-03.pin(h)`.
    Pin,
    /// UNPIN — pop Hash, `S-03.unpin(h)`.
    Unpin,

    // --- Capability (4) ---
    /// CAPHELD — pop CapId, push Bool.
    CapHeld,
    /// ATTENUATE — pop (cap, predicate), push child cap.
    Attenuate,
    /// INVOKE n — pop CapId, then pop n args, invoke the named
    /// operation, push the result. Traps `ENOTHELD` if the CapId
    /// is not in the current `cap_view`.
    Invoke { n: u32 },
    /// REVOKE — pop cap.
    Revoke,

    // --- Weave (2) ---
    /// APPEND — pop entry, push new tip hash.
    Append,
    /// WHY — pop substance hash, push Vec{EntryHash}.
    Why,

    // --- Attention and yield (2) ---
    /// YIELD — seal ExecState as Cont, return control to S-05.
    Yield,
    /// SPLIT — pop budget, push new AttId.
    Split,

    // --- Trap (2) ---
    /// TRAP k — unconditional trap.
    Trap(TrapKind),
    /// ASSERT — pop Bool; trap `EASSERT` if false.
    Assert,

    // --- Reflection (4) ---
    /// SELFHASH — push the current form_hash.
    SelfHash,
    /// PARSEFORM — pop Bytes, push ParsedForm via S-07/parse_form.
    ParseForm,
    /// BINDSLOT — pop (name_hash, form_hash); atomically advance
    /// binding. Traps `EUNAUTHORISED` without kernel mutation cap.
    BindSlot,
    /// READSLOT — pop name_hash, push current form_hash for name.
    ReadSlot,
}

impl Opcode {
    /// Short mnemonic for display / debugging.
    pub fn mnemonic(&self) -> &'static str {
        match self {
            Opcode::Push(_) => "PUSH",
            Opcode::Pop => "POP",
            Opcode::Load(_) => "LOAD",
            Opcode::Store(_) => "STORE",
            Opcode::Add => "ADD",
            Opcode::Sub => "SUB",
            Opcode::Eq => "EQ",
            Opcode::Lt => "LT",
            Opcode::Jmp(_) => "JMP",
            Opcode::Jmpz(_) => "JMPZ",
            Opcode::Call { .. } => "CALL",
            Opcode::CallI { .. } => "CALLI",
            Opcode::Ret => "RET",
            Opcode::MakePair => "MAKEPAIR",
            Opcode::Fst => "FST",
            Opcode::Snd => "SND",
            Opcode::MakeVec(_) => "MAKEVEC",
            Opcode::Seal(_) => "SEAL",
            Opcode::Read => "READ",
            Opcode::Pin => "PIN",
            Opcode::Unpin => "UNPIN",
            Opcode::CapHeld => "CAPHELD",
            Opcode::Attenuate => "ATTENUATE",
            Opcode::Invoke { .. } => "INVOKE",
            Opcode::Revoke => "REVOKE",
            Opcode::Append => "APPEND",
            Opcode::Why => "WHY",
            Opcode::Yield => "YIELD",
            Opcode::Split => "SPLIT",
            Opcode::Trap(_) => "TRAP",
            Opcode::Assert => "ASSERT",
            Opcode::SelfHash => "SELFHASH",
            Opcode::ParseForm => "PARSEFORM",
            Opcode::BindSlot => "BINDSLOT",
            Opcode::ReadSlot => "READSLOT",
        }
    }
}
