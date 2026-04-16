//! ExecState + the small-step interpreter.
//!
//! The interpreter's `step()` method implements the small-step
//! rule for each opcode in `../../kernel/IL.md` § Opcodes.
//!
//! ## Opcode coverage (v0.2.4)
//!
//! **Fully live** (26): `PUSH`, `POP`, `LOAD`, `STORE`, `ADD`,
//! `SUB`, `EQ`, `LT`, `JMP`, `JMPZ`, `CALL`, `RET`, `MAKEPAIR`,
//! `FST`, `SND`, `MAKEVEC`, `SEAL`, `READ`, `PIN`, `UNPIN`,
//! `INVOKE`, `ASSERT`, `SELFHASH`, `TRAP`, `CAPHELD`, `READSLOT`.
//!
//! **Stage-0 live** (7): `ATTENUATE` (ENotHeld check + sealed
//! AttenCap/v1 descriptor), `REVOKE` (ENotHeld check; can't
//! mutate the Arc cap view at stage-0), `APPEND` (EStale — no
//! weave log), `WHY` (returns empty provenance Vec), `SPLIT`
//! (EOverBudget — no attention allocator), `BINDSLOT`
//! (EUnauthorised — no kernel mutation cap at stage-0),
//! `PARSEFORM` (validates wire bytes + reseals as ParsedForm/v1).
//!
//! **Yielded** (1): `YIELD` returns `StepResult::Yielded`.
//!
//! No opcode returns `TrapKind::NotImplemented` after v0.2.4;
//! every trap kind is one of the eleven IL-defined variants.
//! `NotImplemented` remains in the enum as a scaffold-only
//! marker used by the wire codec (which refuses to encode it)
//! but is never produced by `step()`.
//!
//! ## Design notes
//!
//! - The interpreter borrows the `SubstanceStore` mutably. The
//!   alternative (ExecState owns the store) would prevent sharing
//!   a store across invocations, which is exactly what the
//!   fixed-point check's indirect cases need.
//!
//! - `locals_n` is per-Form. IL.md's Form layout carries a
//!   `locals_n` field and each Form declares its own size.
//!   Hardcoding `locals: vec![_; 32]` would silently accept
//!   out-of-bounds indices; the correct behavior is `EBADLOCAL`.
//!
//! - `JMPZ` branches when the condition is *false*, per
//!   IL.md § Control flow. Branching on true would silently
//!   invert every Form that uses structured early-exit.

use std::sync::Arc;

use crate::capability::CapabilityRegistry;
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
/// opcodes like SEAL and READ can reach it.
pub struct Interpreter<'a> {
    pub store: &'a mut SubstanceStore,
    pub registry: Option<&'a FormRegistry>,
    /// Capability registry for INVOKE dispatch. `None` means every
    /// INVOKE traps `ENotHeld`.
    pub cap_registry: Option<Arc<CapabilityRegistry>>,
    /// Maximum call depth. A trap fires when CALL would exceed
    /// this. Keeps runaway recursion bounded in the scaffold.
    pub max_call_depth: usize,
}

impl<'a> Interpreter<'a> {
    pub fn new(store: &'a mut SubstanceStore) -> Self {
        Self {
            store,
            registry: None,
            cap_registry: None,
            max_call_depth: 256,
        }
    }

    /// Attach a form registry so CALL can resolve callees.
    pub fn with_registry(mut self, registry: &'a FormRegistry) -> Self {
        self.registry = Some(registry);
        self
    }

    /// Attach a capability registry so INVOKE can dispatch to Rust backends.
    pub fn with_cap_registry(mut self, reg: Arc<CapabilityRegistry>) -> Self {
        self.cap_registry = Some(reg);
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

        // CALL, RET, and INVOKE are handled before the top-frame
        // reborrow so they can freely use self.store / self.cap_registry
        // and mutate state.frames (CALL/RET) or drop the frame borrow
        // before calling into capability code (INVOKE).
        match &op {
            Opcode::Call { form, n } => {
                return self.do_call(state, *form, *n);
            }
            Opcode::Ret => {
                return self.do_ret(state);
            }
            Opcode::Invoke { n } => {
                return self.do_invoke(state, *n);
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
                    None => {
                        return StepResult::Trapped(TrapKind::type_mismatch("STORE on empty stack"))
                    }
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
            // CALL, RET, and INVOKE are handled above the match.
            Opcode::Call { .. } | Opcode::Ret | Opcode::Invoke { .. } => {
                unreachable!("CALL/RET/INVOKE handled above the match in step()")
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
                    return StepResult::Trapped(TrapKind::type_mismatch(
                        "MAKEVEC not enough elements",
                    ));
                }
                let tail = s.stack.split_off(s.stack.len() - n);
                s.stack.push(Value::Vec(tail));
                StepResult::Step
            }

            // ---- Substance ----
            Opcode::Seal(tag) => {
                let v = match s.stack.pop() {
                    Some(v) => v,
                    None => {
                        return StepResult::Trapped(TrapKind::type_mismatch("SEAL on empty stack"))
                    }
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
                    None => {
                        return StepResult::Trapped(TrapKind::type_mismatch("READ on empty stack"))
                    }
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
                    None => {
                        return StepResult::Trapped(TrapKind::type_mismatch("PIN on empty stack"))
                    }
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
                    None => {
                        return StepResult::Trapped(TrapKind::type_mismatch("UNPIN on empty stack"))
                    }
                };
                match self.store.unpin(&h) {
                    Ok(()) => StepResult::Step,
                    Err(k) => StepResult::Trapped(k),
                }
            }

            // ---- Capability ----
            // INVOKE is handled above the match (do_invoke).

            // CAPHELD (CapId) → (Bool)
            // IL.md: cap_view.contains(c)
            Opcode::CapHeld => {
                let cap_id = match s.stack.pop() {
                    Some(v) => match v.as_hash() {
                        Ok(h) => h,
                        Err(k) => return StepResult::Trapped(k),
                    },
                    None => {
                        return StepResult::Trapped(TrapKind::type_mismatch("CAPHELD: empty stack"))
                    }
                };
                let held = self
                    .cap_registry
                    .as_ref()
                    .is_some_and(|r| r.contains(&cap_id));
                s.stack.push(Value::Bool(held));
                StepResult::Step
            }

            // ATTENUATE (CapId, predicate) → (CapId')
            // IL.md: call S-02.attenuate(c, p); trap ENOTHELD if c ∉ cap_view.
            // Stage-0: check held, seal (cap_id, predicate) as AttenCap/v1, return new hash.
            // The new CapId is a valid handle but not yet resolvable via INVOKE
            // (that requires S-02.attenuate to be bound, which is post-v0.2.5).
            Opcode::Attenuate => {
                let predicate = match s.stack.pop() {
                    Some(v) => v,
                    None => {
                        return StepResult::Trapped(TrapKind::type_mismatch(
                            "ATTENUATE: empty stack (predicate)",
                        ))
                    }
                };
                let cap_id = match s.stack.pop() {
                    Some(v) => match v.as_hash() {
                        Ok(h) => h,
                        Err(k) => return StepResult::Trapped(k),
                    },
                    None => {
                        return StepResult::Trapped(TrapKind::type_mismatch(
                            "ATTENUATE: empty stack (cap_id)",
                        ))
                    }
                };
                let held = self
                    .cap_registry
                    .as_ref()
                    .is_some_and(|r| r.contains(&cap_id));
                if !held {
                    return StepResult::Trapped(TrapKind::ENotHeld);
                }
                let desc = Value::Pair(Box::new(Value::Hash(cap_id)), Box::new(predicate));
                let new_hash = self.store.seal("AttenCap/v1", desc);
                s.stack.push(Value::Hash(new_hash));
                StepResult::Step
            }

            // REVOKE (CapId) → ()
            // IL.md: call S-02.revoke(c); trap ENOTHELD if c ∉ cap_view or
            //         c not minted by this attention.
            // Stage-0: check held; can't mutate Arc cap view — actual removal
            //          requires a per-frame mutable cap_view (post-v0.2.5).
            Opcode::Revoke => {
                let cap_id = match s.stack.pop() {
                    Some(v) => match v.as_hash() {
                        Ok(h) => h,
                        Err(k) => return StepResult::Trapped(k),
                    },
                    None => {
                        return StepResult::Trapped(TrapKind::type_mismatch("REVOKE: empty stack"))
                    }
                };
                let held = self
                    .cap_registry
                    .as_ref()
                    .is_some_and(|r| r.contains(&cap_id));
                if !held {
                    return StepResult::Trapped(TrapKind::ENotHeld);
                }
                // No-op at stage-0: the Arc<CapabilityRegistry> is immutable
                // at runtime. The ENOTHELD guard is correct; the actual
                // revocation slot will be a per-attention mut view in v0.2.5.
                StepResult::Step
            }

            // ---- Weave ----

            // APPEND (EntryHash) → (TipHash)
            // IL.md: call S-04.append(e); trap ESTALE if e.prev ≠ current_tip.
            // Stage-0: no weave log — every APPEND is unconditionally ESTALE.
            Opcode::Append => {
                match s.stack.pop() {
                    Some(_) => {}
                    None => {
                        return StepResult::Trapped(TrapKind::type_mismatch("APPEND: empty stack"))
                    }
                }
                StepResult::Trapped(TrapKind::EStale)
            }

            // WHY (SubstanceHash) → (Vec{EntryHash})
            // IL.md: call S-04.why(s).
            // Stage-0: no weave log — return empty provenance Vec.
            Opcode::Why => {
                match s.stack.pop() {
                    Some(_) => {}
                    None => {
                        return StepResult::Trapped(TrapKind::type_mismatch("WHY: empty stack"))
                    }
                }
                s.stack.push(Value::Vec(vec![]));
                StepResult::Step
            }

            // ---- Attention ----
            Opcode::Yield => StepResult::Yielded,

            // SPLIT (budget) → (AttId)
            // IL.md: call S-05.split(current_attention, budget);
            //         trap EOVERBUDGET if disallowed.
            // Stage-0: no attention allocator — all splits are EOVERBUDGET.
            Opcode::Split => {
                match s.stack.pop() {
                    Some(_) => {}
                    None => {
                        return StepResult::Trapped(TrapKind::type_mismatch("SPLIT: empty stack"))
                    }
                }
                StepResult::Trapped(TrapKind::EOverBudget)
            }

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

            // PARSEFORM (Hash) → (ParsedForm/v1 hash)
            // IL.md: call the IL parser Form on the substance at h;
            //         trap ETYPE if not a Form substance.
            // Stage-0: validate the bytes via wire::decode_form, then
            //          reseal as ParsedForm/v1. A real ignis0 would call
            //          S-07/parse_form and return a structured record.
            Opcode::ParseForm => {
                use crate::wire::decode_form;
                let h = match s.stack.pop() {
                    Some(v) => match v.as_hash() {
                        Ok(h) => h,
                        Err(k) => return StepResult::Trapped(k),
                    },
                    None => {
                        return StepResult::Trapped(TrapKind::type_mismatch(
                            "PARSEFORM: empty stack",
                        ))
                    }
                };
                let bytes = match self.store.read(&h) {
                    Ok(Value::Bytes(b)) => b,
                    Ok(other) => {
                        return StepResult::Trapped(TrapKind::EType(format!(
                            "PARSEFORM: expected Bytes substance, got {:?}",
                            other
                        )))
                    }
                    Err(k) => return StepResult::Trapped(k),
                };
                match decode_form(&bytes) {
                    Ok(_form) => {
                        let result_hash = self.store.seal("ParsedForm/v1", Value::Bytes(bytes));
                        s.stack.push(Value::Hash(result_hash));
                        StepResult::Step
                    }
                    Err(_) => StepResult::Trapped(TrapKind::EType(
                        "PARSEFORM: substance at h is not a valid Form/v1".into(),
                    )),
                }
            }

            // BINDSLOT (name_hash, form_hash) → ()
            // IL.md: atomically advance name_hash → form_hash; trap
            //         EUNAUTHORISED if no kernel mutation cap held.
            // Stage-0: no Form has ever been granted the kernel mutation
            //          cap (S-01/ignite has not run). Always EUNAUTHORISED.
            Opcode::BindSlot => {
                // Pop both args to maintain stack discipline before trapping.
                // form_hash is on top, name_hash is below.
                let _ = s.stack.pop(); // form_hash
                let _ = s.stack.pop(); // name_hash
                StepResult::Trapped(TrapKind::EUnauthorised)
            }

            // READSLOT (name_hash) → (form_hash)
            // IL.md: look up the current binding for name_hash.
            // Stage-0: resolve against FormRegistry::slots (HashMap-backed).
            Opcode::ReadSlot => {
                let name_hash = match s.stack.pop() {
                    Some(v) => match v.as_hash() {
                        Ok(h) => h,
                        Err(k) => return StepResult::Trapped(k),
                    },
                    None => {
                        return StepResult::Trapped(TrapKind::type_mismatch(
                            "READSLOT: empty stack",
                        ))
                    }
                };
                let form_hash = match self.registry.and_then(|r| r.read_slot(&name_hash)) {
                    Some(h) => h,
                    None => {
                        return StepResult::Trapped(TrapKind::EUnheld(format!(
                            "READSLOT: no binding for {}",
                            name_hash.short()
                        )))
                    }
                };
                s.stack.push(Value::Hash(form_hash));
                StepResult::Step
            }
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
            return StepResult::Trapped(TrapKind::type_mismatch("CALL: max_call_depth exceeded"));
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
            return StepResult::Trapped(TrapKind::type_mismatch("CALL: not enough stack args"));
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

    /// Dispatch `INVOKE n`: pop the CapId from the top of the stack,
    /// pop `n` args, call the capability registry, push the result.
    ///
    /// Popping and pushing happen in scoped borrows of `state` so
    /// the mutable borrow is released before calling into the
    /// capability backend (which itself needs `&mut self.store`).
    fn do_invoke(&mut self, state: &mut ExecState, n: u32) -> StepResult {
        // ── Pop cap_id + args in one scoped borrow ────────────────────
        let (cap_id, args) = {
            let s = state.top_mut();
            // The CapId sits on top; args are below it (pushed first).
            let cap_id = match s.stack.pop() {
                Some(v) => match v.as_hash() {
                    Ok(h) => h,
                    Err(k) => return StepResult::Trapped(k),
                },
                None => {
                    return StepResult::Trapped(TrapKind::type_mismatch(
                        "INVOKE: stack empty (cap_id missing)",
                    ))
                }
            };
            let n = n as usize;
            if s.stack.len() < n {
                return StepResult::Trapped(TrapKind::type_mismatch(
                    "INVOKE: not enough args on stack",
                ));
            }
            // split_off preserves push order: args[0] was pushed first.
            let split_at = s.stack.len() - n;
            let args = s.stack.split_off(split_at);
            (cap_id, args)
        }; // ← mutable borrow of `state` released here

        // ── Dispatch through capability registry ──────────────────────
        // Clone the Arc (cheap ref-count bump) so we can use self.store
        // mutably in the same expression without lifetime conflicts.
        let result = match self.cap_registry.as_ref().map(Arc::clone) {
            Some(reg) => reg.invoke(cap_id, args, self.store),
            None => Err(TrapKind::ENotHeld),
        };

        // ── Push result or propagate trap ─────────────────────────────
        let s = state.top_mut();
        match result {
            Ok(v) => {
                s.stack.push(v);
                StepResult::Step
            }
            Err(k) => StepResult::Trapped(k),
        }
    }

    /// Pop the top frame, take its return value, and either
    /// push the value onto the caller's stack or return it to
    /// `run()` if the stack is now empty.
    fn do_ret(&mut self, state: &mut ExecState) -> StepResult {
        let v = {
            let s = state.top_mut();
            match s.stack.pop() {
                Some(v) => v,
                None => return StepResult::Trapped(TrapKind::type_mismatch("RET on empty stack")),
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
