//! ExecState + the small-step interpreter.
//!
//! The interpreter's `step()` method implements the small-step
//! rule for each opcode in `../../kernel/IL.md` § Opcodes. Most
//! opcodes are stubbed with `TrapKind::NotImplemented` for the
//! scaffold; the ones `F` uses (`STORE`, `LOAD`, `PUSH`, `ADD`,
//! `RET`) are implemented end-to-end so the A9.3 fixed-point
//! check's direct case produces a real verdict.

use crate::opcode::Opcode;
use crate::store::SubstanceStore;
use crate::value::{Hash, TrapKind, Value};

/// Runtime state of one Form invocation.
#[derive(Debug)]
pub struct ExecState {
    pub form_hash: Hash,
    pub pc: usize,
    pub stack: Vec<Value>,
    pub locals: Vec<Value>,
    pub code: Vec<Opcode>,
}

impl ExecState {
    /// Build a fresh ExecState for executing `code` under
    /// `form_hash` with the given initial inputs pushed onto
    /// the stack in reverse (so the first argument is on top).
    pub fn new(form_hash: Hash, code: Vec<Opcode>, locals_n: usize, inputs: Vec<Value>) -> Self {
        let mut stack = Vec::with_capacity(inputs.len() + 4);
        for v in inputs.into_iter().rev() {
            stack.push(v);
        }
        ExecState {
            form_hash,
            pc: 0,
            stack,
            locals: vec![Value::Unit; locals_n],
            code,
        }
    }
}

/// The three possible outcomes of a single step.
#[derive(Debug)]
pub enum StepResult {
    /// Continue executing; pc has been advanced.
    Step,
    /// The Form returned a value.
    Returned(Value),
    /// The Form trapped.
    Trapped(TrapKind),
    /// The Form yielded (not used in the fixed-point scaffold).
    Yielded,
}

/// The three possible outcomes of a full invocation.
#[derive(Debug)]
pub enum ExecVerdict {
    Returned(Value),
    Trapped(TrapKind),
    Yielded,
}

/// The interpreter. Holds a reference to a store so opcodes
/// like SEAL and READ can reach it. In this scaffold the
/// interpreter does not hold a weave or a cap registry; opcodes
/// that need them return `NotImplemented`.
pub struct Interpreter<'a> {
    pub store: &'a mut SubstanceStore,
}

impl<'a> Interpreter<'a> {
    pub fn new(store: &'a mut SubstanceStore) -> Self {
        Self { store }
    }

    /// Run an ExecState to a verdict. Bounded by `max_steps`
    /// to catch infinite loops in scaffold code paths.
    pub fn run(&mut self, state: &mut ExecState, max_steps: usize) -> ExecVerdict {
        for _ in 0..max_steps {
            match self.step(state) {
                StepResult::Step => continue,
                StepResult::Returned(v) => return ExecVerdict::Returned(v),
                StepResult::Trapped(k) => return ExecVerdict::Trapped(k),
                StepResult::Yielded => return ExecVerdict::Yielded,
            }
        }
        ExecVerdict::Trapped(TrapKind::NotImplemented)
    }

    /// Execute one instruction.
    pub fn step(&mut self, s: &mut ExecState) -> StepResult {
        if s.pc >= s.code.len() {
            return StepResult::Trapped(TrapKind::ETYPE);
        }
        let op = s.code[s.pc].clone();
        s.pc += 1;

        match op {
            // ---- Stack and locals ----
            Opcode::Push(v) => {
                s.stack.push(v);
                StepResult::Step
            }
            Opcode::Pop => {
                if s.stack.pop().is_none() {
                    return StepResult::Trapped(TrapKind::ETYPE);
                }
                StepResult::Step
            }
            Opcode::Load(i) => {
                let idx = i as usize;
                if idx >= s.locals.len() {
                    return StepResult::Trapped(TrapKind::EBADLOCAL);
                }
                s.stack.push(s.locals[idx].clone());
                StepResult::Step
            }
            Opcode::Store(i) => {
                let idx = i as usize;
                if idx >= s.locals.len() {
                    return StepResult::Trapped(TrapKind::EBADLOCAL);
                }
                let v = match s.stack.pop() {
                    Some(v) => v,
                    None => return StepResult::Trapped(TrapKind::ETYPE),
                };
                s.locals[idx] = v;
                StepResult::Step
            }

            // ---- Arithmetic and comparison ----
            Opcode::Add => binary_nat(s, |a, b| Value::Nat(a.wrapping_add(b))),
            Opcode::Sub => {
                let b = match pop_nat(s) {
                    Ok(v) => v,
                    Err(k) => return StepResult::Trapped(k),
                };
                let a = match pop_nat(s) {
                    Ok(v) => v,
                    Err(k) => return StepResult::Trapped(k),
                };
                if b > a {
                    return StepResult::Trapped(TrapKind::EUNDERFLOW);
                }
                s.stack.push(Value::Nat(a - b));
                StepResult::Step
            }
            Opcode::Eq => {
                let b = match s.stack.pop() {
                    Some(v) => v,
                    None => return StepResult::Trapped(TrapKind::ETYPE),
                };
                let a = match s.stack.pop() {
                    Some(v) => v,
                    None => return StepResult::Trapped(TrapKind::ETYPE),
                };
                s.stack.push(Value::Bool(a == b));
                StepResult::Step
            }
            Opcode::Lt => binary_nat(s, |a, b| Value::Bool(a < b)),

            // ---- Control flow ----
            Opcode::Jmp(off) => {
                let new_pc = (s.pc as isize) + (off as isize);
                if new_pc < 0 || new_pc as usize > s.code.len() {
                    return StepResult::Trapped(TrapKind::ETYPE);
                }
                s.pc = new_pc as usize;
                StepResult::Step
            }
            Opcode::Jmpz(off) => {
                let b = match s.stack.pop() {
                    Some(Value::Bool(b)) => b,
                    _ => return StepResult::Trapped(TrapKind::ETYPE),
                };
                if !b {
                    let new_pc = (s.pc as isize) + (off as isize);
                    if new_pc < 0 || new_pc as usize > s.code.len() {
                        return StepResult::Trapped(TrapKind::ETYPE);
                    }
                    s.pc = new_pc as usize;
                }
                StepResult::Step
            }
            Opcode::Call { .. } => StepResult::Trapped(TrapKind::NotImplemented),
            Opcode::Ret => {
                let v = match s.stack.pop() {
                    Some(v) => v,
                    None => return StepResult::Trapped(TrapKind::ETYPE),
                };
                StepResult::Returned(v)
            }

            // ---- Structure ----
            Opcode::MakePair => {
                let b = match s.stack.pop() {
                    Some(v) => v,
                    None => return StepResult::Trapped(TrapKind::ETYPE),
                };
                let a = match s.stack.pop() {
                    Some(v) => v,
                    None => return StepResult::Trapped(TrapKind::ETYPE),
                };
                s.stack.push(Value::Pair(Box::new(a), Box::new(b)));
                StepResult::Step
            }
            Opcode::Fst => match s.stack.pop() {
                Some(Value::Pair(a, _)) => {
                    s.stack.push(*a);
                    StepResult::Step
                }
                _ => StepResult::Trapped(TrapKind::ETYPE),
            },
            Opcode::Snd => match s.stack.pop() {
                Some(Value::Pair(_, b)) => {
                    s.stack.push(*b);
                    StepResult::Step
                }
                _ => StepResult::Trapped(TrapKind::ETYPE),
            },
            Opcode::MakeVec(n) => {
                let n = n as usize;
                if s.stack.len() < n {
                    return StepResult::Trapped(TrapKind::ETYPE);
                }
                let tail = s.stack.split_off(s.stack.len() - n);
                s.stack.push(Value::Vec(tail));
                StepResult::Step
            }

            // ---- Substance ----
            Opcode::Seal(tag) => {
                let v = match s.stack.pop() {
                    Some(v) => v,
                    None => return StepResult::Trapped(TrapKind::ETYPE),
                };
                let h = self.store.seal(&tag, v);
                s.stack.push(Value::Hash(h));
                StepResult::Step
            }
            Opcode::Read => {
                let h = match s.stack.pop() {
                    Some(Value::Hash(h)) => h,
                    _ => return StepResult::Trapped(TrapKind::ETYPE),
                };
                match self.store.read(&h) {
                    Ok(v) => {
                        s.stack.push(v);
                        StepResult::Step
                    }
                    Err(k) => StepResult::Trapped(k),
                }
            }
            Opcode::Pin => {
                let h = match s.stack.pop() {
                    Some(Value::Hash(h)) => h,
                    _ => return StepResult::Trapped(TrapKind::ETYPE),
                };
                match self.store.pin(&h) {
                    Ok(()) => StepResult::Step,
                    Err(k) => StepResult::Trapped(k),
                }
            }
            Opcode::Unpin => {
                let h = match s.stack.pop() {
                    Some(Value::Hash(h)) => h,
                    _ => return StepResult::Trapped(TrapKind::ETYPE),
                };
                match self.store.unpin(&h) {
                    Ok(()) => StepResult::Step,
                    Err(k) => StepResult::Trapped(k),
                }
            }

            // ---- Capability (stubbed) ----
            Opcode::CapHeld
            | Opcode::Attenuate
            | Opcode::Invoke
            | Opcode::Revoke => StepResult::Trapped(TrapKind::NotImplemented),

            // ---- Weave (stubbed) ----
            Opcode::Append | Opcode::Why => StepResult::Trapped(TrapKind::NotImplemented),

            // ---- Attention (stubbed) ----
            Opcode::Yield => StepResult::Yielded,
            Opcode::Split => StepResult::Trapped(TrapKind::NotImplemented),

            // ---- Trap ----
            Opcode::Trap(k) => StepResult::Trapped(k),
            Opcode::Assert => match s.stack.pop() {
                Some(Value::Bool(true)) => StepResult::Step,
                Some(Value::Bool(false)) => StepResult::Trapped(TrapKind::EASSERT),
                _ => StepResult::Trapped(TrapKind::ETYPE),
            },

            // ---- Reflection (stubbed) ----
            Opcode::SelfHash => {
                s.stack.push(Value::Hash(s.form_hash));
                StepResult::Step
            }
            Opcode::ParseForm | Opcode::BindSlot | Opcode::ReadSlot => {
                StepResult::Trapped(TrapKind::NotImplemented)
            }
        }
    }
}

// --- helpers ---

fn pop_nat(s: &mut ExecState) -> Result<u64, TrapKind> {
    match s.stack.pop() {
        Some(v) => v.as_nat(),
        None => Err(TrapKind::ETYPE),
    }
}

fn binary_nat(s: &mut ExecState, f: impl Fn(u64, u64) -> Value) -> StepResult {
    let b = match pop_nat(s) {
        Ok(v) => v,
        Err(k) => return StepResult::Trapped(k),
    };
    let a = match pop_nat(s) {
        Ok(v) => v,
        Err(k) => return StepResult::Trapped(k),
    };
    s.stack.push(f(a, b));
    StepResult::Step
}
