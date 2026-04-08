//! ExecState + the small-step interpreter.
//!
//! The interpreter's `step()` method implements the small-step
//! rule for each opcode in `../../kernel/IL.md` § Opcodes. Most
//! opcodes are stubbed with `TrapKind::NotImplemented` for the
//! scaffold; the ones `F` uses (`STORE`, `LOAD`, `PUSH`, `ADD`,
//! `RET`) are implemented end-to-end so the A9.3 fixed-point
//! check's direct case produces a real verdict.
//!
//! Design notes:
//!
//! - The interpreter borrows the `SubstanceStore` mutably. The
//!   alternative (ExecState owns the store) would prevent
//!   sharing a store across invocations, which is exactly
//!   what the fixed-point check's indirect cases need.
//!
//! - `locals_n` is per-Form. IL.md's Form layout carries a
//!   `locals_n` field and each Form declares its own size.
//!   Hardcoding `locals: vec![_; 32]` would make the
//!   interpreter silently accept locals-index overflow in
//!   Forms that declared fewer slots; the structural behaviour
//!   must be `EBADLOCAL` on out-of-bounds.
//!
//! - `JMPZ` branches when the condition is *false*, per
//!   IL.md § Control flow. Branching on true would silently
//!   invert the semantics of every Form that uses structured
//!   early-exit.

use crate::opcode::Opcode;
use crate::registry::FormRegistry;
use crate::store::SubstanceStore;
use crate::value::{Hash, TrapKind, Value};

/// A single activation frame. Each CALL pushes one of these;
/// each RET pops one. The topmost frame is the one `step()`
/// operates on — its fields are mirrored directly into the
/// `ExecState` wrapper's public getters so existing callers
/// that read `state.pc`, `state.stack`, etc. continue to see
/// the current frame.
#[derive(Debug, Clone)]
pub struct Frame {
    pub form_hash: Hash,
    pub pc: usize,
    pub stack: Vec<Value>,
    pub locals: Vec<Value>,
    pub code: Vec<Opcode>,
}

/// Runtime state of a running Form stack.
///
/// Invariant: `frames` is non-empty while execution is in
/// progress. The top frame (`frames.last()`) is the currently
/// executing activation. On RET the top frame is popped and
/// its return value is pushed onto the caller's stack; when
/// the final frame is popped, `run()` returns that value.
#[derive(Debug)]
pub struct ExecState {
    pub frames: Vec<Frame>,
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
            frames: vec![Frame {
                form_hash,
                pc: 0,
                stack,
                locals: vec![Value::Unit; locals_n],
                code,
            }],
        }
    }

    /// Topmost frame, panics if empty (violated invariant).
    pub fn top(&self) -> &Frame {
        self.frames.last().expect("ExecState frames empty")
    }

    pub fn top_mut(&mut self) -> &mut Frame {
        self.frames.last_mut().expect("ExecState frames empty")
    }

    pub fn depth(&self) -> usize {
        self.frames.len()
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

/// The interpreter. Holds a mutable reference to a store so
/// opcodes like SEAL and READ can reach it. In this scaffold
/// the interpreter does not hold a weave or a cap registry;
/// opcodes that need them return `NotImplemented`.
pub struct Interpreter<'a> {
    pub store: &'a mut SubstanceStore,
    pub registry: Option<&'a FormRegistry>,
    /// Maximum call depth. A trap fires when CALL would exceed
    /// this. Keeps runaway recursion bounded in the scaffold.
    pub max_call_depth: usize,
}

impl<'a> Interpreter<'a> {
    pub fn new(store: &'a mut SubstanceStore) -> Self {
        Self {
            store,
            registry: None,
            max_call_depth: 256,
        }
    }

    /// Attach a form registry so CALL can resolve callees.
    pub fn with_registry(mut self, registry: &'a FormRegistry) -> Self {
        self.registry = Some(registry);
        self
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
        ExecVerdict::Trapped(TrapKind::NotImplemented("max_steps exceeded".into()))
    }

    /// Execute one instruction on the topmost frame.
    pub fn step(&mut self, state: &mut ExecState) -> StepResult {
        // Fetch the next opcode in a scoped borrow, then release
        // the borrow so arms like CALL/RET can mutate the frame
        // stack itself.
        let op = {
            let s = state.top_mut();
            if s.pc >= s.code.len() {
                return StepResult::Trapped(TrapKind::type_mismatch("pc past end of code"));
            }
            let op = s.code[s.pc].clone();
            s.pc += 1;
            op
        };

        // CALL and RET mutate state.frames; handle them before
        // reborrowing the top frame.
        match &op {
            Opcode::Call { form, n } => {
                return self.do_call(state, *form, *n);
            }
            Opcode::Ret => {
                return self.do_ret(state);
            }
            _ => {}
        }

        let s = state.top_mut();

        match op {
            // ---- Stack and locals ----
            Opcode::Push(v) => {
                s.stack.push(v);
                StepResult::Step
            }
            Opcode::Pop => {
                if s.stack.pop().is_none() {
                    return StepResult::Trapped(TrapKind::type_mismatch("POP on empty stack"));
                }
                StepResult::Step
            }
            Opcode::Load(i) => {
                let idx = i as usize;
                if idx >= s.locals.len() {
                    return StepResult::Trapped(TrapKind::bad_local(i));
                }
                s.stack.push(s.locals[idx].clone());
                StepResult::Step
            }
            Opcode::Store(i) => {
                let idx = i as usize;
                if idx >= s.locals.len() {
                    return StepResult::Trapped(TrapKind::bad_local(i));
                }
                let v = match s.stack.pop() {
                    Some(v) => v,
                    None => return StepResult::Trapped(TrapKind::type_mismatch("STORE on empty stack")),
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
                    return StepResult::Trapped(TrapKind::EUnderflow);
                }
                s.stack.push(Value::Nat(a - b));
                StepResult::Step
            }
            Opcode::Eq => {
                let b = match s.stack.pop() {
                    Some(v) => v,
                    None => return StepResult::Trapped(TrapKind::type_mismatch("EQ a")),
                };
                let a = match s.stack.pop() {
                    Some(v) => v,
                    None => return StepResult::Trapped(TrapKind::type_mismatch("EQ b")),
                };
                s.stack.push(Value::Bool(a == b));
                StepResult::Step
            }
            Opcode::Lt => binary_nat(s, |a, b| Value::Bool(a < b)),

            // ---- Control flow ----
            Opcode::Jmp(off) => {
                let new_pc = (s.pc as isize) + (off as isize);
                if new_pc < 0 || new_pc as usize > s.code.len() {
                    return StepResult::Trapped(TrapKind::type_mismatch("JMP out of bounds"));
                }
                s.pc = new_pc as usize;
                StepResult::Step
            }
            // IL.md says: "JMPZ off — (b) → () — branch if `b = false`".
            // We pop a Bool and branch when it is FALSE, per the spec.
            Opcode::Jmpz(off) => {
                let b = match s.stack.pop() {
                    Some(Value::Bool(b)) => b,
                    _ => return StepResult::Trapped(TrapKind::type_mismatch("JMPZ expected Bool")),
                };
                if !b {
                    let new_pc = (s.pc as isize) + (off as isize);
                    if new_pc < 0 || new_pc as usize > s.code.len() {
                        return StepResult::Trapped(TrapKind::type_mismatch("JMPZ target oob"));
                    }
                    s.pc = new_pc as usize;
                }
                StepResult::Step
            }
            // CALL and RET are handled above the match, before
            // the top-frame borrow is re-acquired.
            Opcode::Call { .. } | Opcode::Ret => {
                unreachable!("CALL/RET handled above the match in step()")
            }

            // ---- Structure ----
            Opcode::MakePair => {
                let b = match s.stack.pop() {
                    Some(v) => v,
                    None => return StepResult::Trapped(TrapKind::type_mismatch("MAKEPAIR b")),
                };
                let a = match s.stack.pop() {
                    Some(v) => v,
                    None => return StepResult::Trapped(TrapKind::type_mismatch("MAKEPAIR a")),
                };
                s.stack.push(Value::Pair(Box::new(a), Box::new(b)));
                StepResult::Step
            }
            Opcode::Fst => match s.stack.pop() {
                Some(Value::Pair(a, _)) => {
                    s.stack.push(*a);
                    StepResult::Step
                }
                _ => StepResult::Trapped(TrapKind::type_mismatch("FST expected Pair")),
            },
            Opcode::Snd => match s.stack.pop() {
                Some(Value::Pair(_, b)) => {
                    s.stack.push(*b);
                    StepResult::Step
                }
                _ => StepResult::Trapped(TrapKind::type_mismatch("SND expected Pair")),
            },
            Opcode::MakeVec(n) => {
                let n = n as usize;
                if s.stack.len() < n {
                    return StepResult::Trapped(TrapKind::type_mismatch("MAKEVEC not enough elements"));
                }
                let tail = s.stack.split_off(s.stack.len() - n);
                s.stack.push(Value::Vec(tail));
                StepResult::Step
            }

            // ---- Substance ----
            Opcode::Seal(tag) => {
                let v = match s.stack.pop() {
                    Some(v) => v,
                    None => return StepResult::Trapped(TrapKind::type_mismatch("SEAL on empty stack")),
                };
                let h = self.store.seal(&tag, v);
                s.stack.push(Value::Hash(h));
                StepResult::Step
            }
            Opcode::Read => {
                let h = match s.stack.pop() {
                    Some(v) => match v.as_hash() {
                        Ok(h) => h,
                        Err(k) => return StepResult::Trapped(k),
                    },
                    None => return StepResult::Trapped(TrapKind::type_mismatch("READ on empty stack")),
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
                    Some(v) => match v.as_hash() {
                        Ok(h) => h,
                        Err(k) => return StepResult::Trapped(k),
                    },
                    None => return StepResult::Trapped(TrapKind::type_mismatch("PIN on empty stack")),
                };
                match self.store.pin(&h) {
                    Ok(()) => StepResult::Step,
                    Err(k) => StepResult::Trapped(k),
                }
            }
            Opcode::Unpin => {
                let h = match s.stack.pop() {
                    Some(v) => match v.as_hash() {
                        Ok(h) => h,
                        Err(k) => return StepResult::Trapped(k),
                    },
                    None => return StepResult::Trapped(TrapKind::type_mismatch("UNPIN on empty stack")),
                };
                match self.store.unpin(&h) {
                    Ok(()) => StepResult::Step,
                    Err(k) => StepResult::Trapped(k),
                }
            }

            // ---- Capability (stubbed) ----
            Opcode::CapHeld => StepResult::Trapped(TrapKind::NotImplemented("CAPHELD".into())),
            Opcode::Attenuate => StepResult::Trapped(TrapKind::NotImplemented("ATTENUATE".into())),
            Opcode::Invoke => StepResult::Trapped(TrapKind::NotImplemented("INVOKE".into())),
            Opcode::Revoke => StepResult::Trapped(TrapKind::NotImplemented("REVOKE".into())),

            // ---- Weave (stubbed) ----
            Opcode::Append => StepResult::Trapped(TrapKind::NotImplemented("APPEND".into())),
            Opcode::Why => StepResult::Trapped(TrapKind::NotImplemented("WHY".into())),

            // ---- Attention (stubbed) ----
            Opcode::Yield => StepResult::Yielded,
            Opcode::Split => StepResult::Trapped(TrapKind::NotImplemented("SPLIT".into())),

            // ---- Trap ----
            Opcode::Trap(k) => StepResult::Trapped(k),
            Opcode::Assert => match s.stack.pop() {
                Some(Value::Bool(true)) => StepResult::Step,
                Some(Value::Bool(false)) => StepResult::Trapped(TrapKind::EAssert),
                _ => StepResult::Trapped(TrapKind::type_mismatch("ASSERT expected Bool")),
            },

            // ---- Reflection ----
            Opcode::SelfHash => {
                s.stack.push(Value::Hash(s.form_hash));
                StepResult::Step
            }
            Opcode::ParseForm => StepResult::Trapped(TrapKind::NotImplemented("PARSEFORM".into())),
            Opcode::BindSlot => StepResult::Trapped(TrapKind::NotImplemented("BINDSLOT".into())),
            Opcode::ReadSlot => StepResult::Trapped(TrapKind::NotImplemented("READSLOT".into())),
        }
    }
}

impl<'a> Interpreter<'a> {
    /// Resolve `form` in the attached registry, push a new frame
    /// onto `state.frames`, and move the top `n` stack values
    /// from the caller's stack into the callee's stack as its
    /// initial inputs (in order — arg0 ends up on top).
    fn do_call(&mut self, state: &mut ExecState, form: Hash, n: u32) -> StepResult {
        if state.frames.len() >= self.max_call_depth {
            return StepResult::Trapped(TrapKind::type_mismatch(
                "CALL: max_call_depth exceeded",
            ));
        }
        let registry = match self.registry {
            Some(r) => r,
            None => {
                return StepResult::Trapped(TrapKind::NotImplemented(
                    "CALL: no FormRegistry attached to Interpreter".into(),
                ))
            }
        };
        let loaded = match registry.get(&form) {
            Some(lf) => lf.clone(),
            None => {
                return StepResult::Trapped(TrapKind::EUnheld(format!(
                    "CALL: no Form at {}",
                    form.short()
                )))
            }
        };
        let n = n as usize;
        // Pop the n args from the caller's stack. IL.md says CALL
        // consumes n arguments from the caller's stack; we
        // preserve their order so the callee sees arg0 on top.
        let caller = state.top_mut();
        if caller.stack.len() < n {
            return StepResult::Trapped(TrapKind::type_mismatch(
                "CALL: not enough stack args",
            ));
        }
        let split_at = caller.stack.len() - n;
        let callee_stack: Vec<Value> = caller.stack.split_off(split_at);
        // split_off preserves order so the deepest caller arg is
        // at index 0; to match `ExecState::new`'s "arg0 on top"
        // convention we treat these as already in the desired
        // order — caller pushed arg_{n-1}, ..., arg_0, so the
        // popped region ends with arg_0 on top.
        state.frames.push(Frame {
            form_hash: form,
            pc: 0,
            stack: callee_stack,
            locals: vec![Value::Unit; loaded.locals_n],
            code: loaded.code,
        });
        StepResult::Step
    }

    /// Pop the top frame, take its return value, and either
    /// push the value onto the caller's stack or return it to
    /// `run()` if the stack is now empty.
    fn do_ret(&mut self, state: &mut ExecState) -> StepResult {
        let v = {
            let s = state.top_mut();
            match s.stack.pop() {
                Some(v) => v,
                None => {
                    return StepResult::Trapped(TrapKind::type_mismatch(
                        "RET on empty stack",
                    ))
                }
            }
        };
        state.frames.pop();
        if state.frames.is_empty() {
            StepResult::Returned(v)
        } else {
            state.top_mut().stack.push(v);
            StepResult::Step
        }
    }
}

// --- helpers ---

fn pop_nat(s: &mut Frame) -> Result<u128, TrapKind> {
    match s.stack.pop() {
        Some(v) => v.as_nat(),
        None => Err(TrapKind::type_mismatch("expected Nat, stack empty")),
    }
}

fn binary_nat(s: &mut Frame, f: impl Fn(u128, u128) -> Value) -> StepResult {
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
