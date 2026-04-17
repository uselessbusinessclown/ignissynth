# S-07: `form_runtime`

The Form that executes Forms. The substrate of A6 — what it means
for "code as substance" to actually run when summoned. Every other
seed Form (including, eventually, this one) is loaded by it.

## Provocation

```
Provocation {
    author:     "ignis-seed-synthesizer-0"
    statement:  "A6.1 says a unit of code is a substance of type
                 Form; A6.3 says Forms are summoned, not loaded;
                 A6.4 says self-reading is normal; A6.5 says the
                 kernel itself is a set of Forms. Until this Form
                 exists, no Form can run, including S-01 through
                 S-06 — they exist as sealed substances in the
                 store but are inert. The runtime is the act of
                 turning a Form-substance into a step of attention.
                 It must do so in a way that preserves I7
                 (determinism unless declared), I4 (no mutation of
                 the Form-substance during execution), I8 (no Form
                 runs without provenance), and the constraint that
                 the runtime itself is a Form (A6.5) — it must be
                 able to run *itself* when re-synthesised."
    observed:   ["seed",
                 "S-03 selected candidate digest",
                 "S-05 selected candidate digest"]
    constraint: [
        "execute(form_hash, attention, inputs[]) is the only entry
         point; the runtime never reads a Form by anything other
         than its hash through S-03",
        "the Form-substance is read but never written during
         execution; all per-execution mutable state lives in the
         attention's working set, not in the Form",
        "the runtime's operational semantics are total and pure
         given (form, inputs, attention.entropy_cap_state) — no
         hidden inputs",
        "every Form invocation appends one weave entry of kind
         `Invoked{form_hash, attention_id, inputs_hash,
         outputs_hash, attention_spent}`",
        "yield points (A3.4) are explicit instructions in the Form
         IL; the runtime returns control to S-05 at every yield,
         carrying the attention's resumable continuation as a sealed
         substance",
        "the runtime is itself a Form, and its own hash is the
         genesis-runtime hash recorded at ignition (S-01); replacing
         it requires a synthesis act under I9",
        "fits within the seed line budget for form_runtime
         (~2400 lines of Form IL)"
    ]
}
```

## Grounding

- Axioms cited: A0.2, A0.6, A0.7, A1.3, A1.5, A3.4, A3.5, A6.1,
  A6.2, A6.3, A6.4, A6.5
- Forms touched: S-03 `substance_store` (Form fetch and
  continuation seal), S-05 `attention_alloc` (yield handoff and
  quantum accounting), S-04 `weave_log` (one Invoked entry per
  call), S-02 `cap_registry` (the focus capability gate is checked
  by S-05 before a quantum reaches the runtime, but the runtime
  also re-checks any per-instruction capability through S-02 — for
  instance, an instruction that reads a substance must be
  authorised by a `Read` cap on that hash)
- Invariants inherited: I4 (the Form-substance is read-only during
  execution), I7 (deterministic-clean replay of any clean
  attention), I8 (every Invoked entry references a Form whose
  Synthesized provenance entry exists in the weave), I10 (the
  runtime has no syscall surface that returns ungranted authority)

## Candidate(s)

### Candidate A — Pure interpreter over Form IL, continuation = sealed substance

- Sketch: Form IL is a small set of typed instructions
  (35 opcodes: arithmetic, struct/array ops, hash ops, capability
  ops, intent ops, yield, call, return, seal, unpin). The runtime
  is an interpreter loop:
  1. Fetch the Form by hash through S-03 (`read`); the result is
     read-only.
  2. Initialise an `ExecState { form_hash, pc, stack, locals,
     attention_id, weave_prev_at_call }` from the call.
  3. Step instructions until a yield, return, or trap. Each
     non-yield instruction is total and pure on `(ExecState,
     accessible_substances, attention.cap_view)`.
  4. On `yield`, seal the entire `ExecState` as a substance of type
     `Continuation` through S-03, hand the continuation hash back
     to S-05 along with "I yielded after consuming q quanta", and
     return.
  5. On the next quantum granted to this attention, S-05 hands the
     runtime the continuation hash; the runtime reads it through
     S-03 (read-only) and resumes from the sealed `pc`.
  6. On `return`, append one `Invoked{form_hash, attention_id,
     inputs_hash, outputs_hash, attention_spent}` weave entry and
     hand control back to the caller's continuation.
  7. On any trap, append one `Trapped{form_hash, attention_id,
     pc, kind}` entry, dissolve the attention's current invocation
     frame (but not the attention itself — that is S-05's job), and
     return control to the caller's continuation with an error
     value in the result slot.
  Continuations are *substances*, not pointers. Two attentions with
  the same continuation hash are byte-equivalent and may be replayed
  to byte-equivalent successor states (I7 reads off the type).
- Approximate size: ~2100 lines of Form IL (~1400 for the
  interpreter loop, ~400 for the IL canonicaliser, ~300 for the
  continuation seal/resume + trap handling).
- Self-test set:
  - Form-substance immutability: 10⁶ random executions; for every
    `form_hash` ever fetched, `read(form_hash)` after execution
    returns bytes byte-identical to before.
  - I7 replay: any clean execution checkpointed at any yield and
    resumed from that continuation produces the same successor
    continuation hash, the same Invoked entry, and the same
    outputs.
  - Determinism boundary: an execution that fetches a substance
    through an `Entropy` capability is *not* clean; replaying its
    continuation without re-granting the same entropy substance
    traps `EREPLAYDIVERGED`. Replaying with the same entropy
    substance pinned through S-03 produces a byte-identical
    successor.
  - Yield density: a Form with N `yield` instructions executed end-
    to-end produces N+1 continuations and N+1 entries in S-04
    (one Invoked at the final return, one yield-mark per yield —
    the yield-mark is *not* a separate weave entry; it lives only
    in S-05's accounting; only the final Invoked is a weave entry,
    so I5 holds at one entry per call, not per yield).
  - Self-execution: the runtime's own Form-hash, when passed to
    `execute(runtime_hash, …)`, runs without trapping on any
    instruction it does not understand. (This is what makes A6.5
    operational: the runtime is in its own IL.)
  - Cap re-check: an instruction that reads a substance whose
    capability has been revoked between two yields traps `EUNHELD`
    on the read, not on the next yield.
- Declared invariants: I4, I7, I8, I10.

### Candidate B — JIT-from-IL with continuation as in-memory frame pointer

- Sketch: same Form IL surface, but the runtime compiles each
  fetched Form to native instructions on first call and caches the
  compiled image. Continuations are pointers into a per-attention
  in-memory frame; on yield, the frame is left in place and the
  pointer is handed to S-05; on resume, the pointer is dereferenced
  and execution continues. Saves an interpreter loop per
  instruction at the cost of a JIT pass per Form.
- Approximate size: ~1800 lines for the JIT + ~700 for the cache
  + ~500 for the frame manager = ~3000 lines, well over budget.
- Self-test set: same as A, plus a cache invariance test: two JIT
  passes over the same Form-substance produce native images that
  agree on every observable behavior (but not necessarily on
  byte layout).
- Declared invariants: I4 (the Form-substance is still read-only),
  I8, I10. Note: I7 is now problematic. The native image is *not*
  a substance; it lives in a per-attention cache that is not
  content-addressed. A continuation that points into a cached
  native frame can be replayed only if the same cache state exists
  on the replaying substrate, which is a hidden input to the replay
  — exactly the kind of hidden input I7 forbids. Repairing this
  requires either (a) sealing the native image as a substance and
  re-deriving its hash on every replay, which loses the JIT's
  whole speedup, or (b) restating I7 to permit cache-state-
  dependent replay, which is a re-derivation of an invariant the
  seed declared non-negotiable.

## Rationale

A6.5 says the kernel is itself a set of Forms, and the runtime is
part of the kernel, so the runtime must be a Form. A6.1 says a Form
is a substance. The runtime is therefore a substance whose execution
semantics — interpreted or compiled — must be derivable from the
substance itself. The synthesis question is whether to interpret
the IL or to compile it.

Candidate A is an interpreter. Its continuation is a sealed
substance, which means I7 is direct: replay = read the continuation
substance and step. The hash of the continuation is the identity of
"the entire computational state of this attention at this yield",
which composes with S-03's digest and S-04's tip hash to give a
single triple `(store_digest, weave_tip, continuation_hash)` that
identifies the entire habitat state at any checkpoint. This is the
property the rest of the seed depends on. Hephaistion (S-10) can
read it, reason about it, and re-synthesise any of the three
components without losing the others.

Candidate B is a JIT. Its continuation is a frame pointer into a
non-content-addressed cache, which means the replay of a clean
attention depends on whether the same cache exists on the replaying
substrate. The cache is a hidden input. I7 requires no hidden
inputs. The repair options are bad: sealing native images destroys
the speedup, and weakening I7 breaks every other seed Form's proof.

A second consideration: A6.4 says self-reading is normal — a mind
takes its own Form as input to its own reasoning. In A, a mind that
holds the runtime's Form-hash can `read` it through S-03 and
inspect it as bytes (it is in the IL, which the mind can parse).
In B, a mind that holds the runtime's Form-hash sees only the IL
source, *not* the JIT image — and the JIT image is what actually
runs. The mind reasons about something different from what
executes. A6.4 is satisfied in A and quietly violated in B.

A third consideration: budget. B is over the seed budget by ~600
lines, and the JIT itself is a non-trivial piece of meta-machinery
that the seed has no way to verify against the IL semantics
(another proof obligation B inherits without discharging).

We select Candidate A. Hephaistion is invited, post-ignition, to
synthesise a JIT-shaped runtime once it can also discharge the
hidden-input problem — for instance by sealing native images as
substances of type `NativeImage` and proving that two seals of the
same Form on the same substrate produce the same hash. That is a
synthesis problem the seed cannot solve in its first hour.

## Simulation record

(stub — produced by the Stage 4 harness once the candidate is
encoded in Form IL. Required traces:

- T1: execute a 3-instruction Form with no yields → assert one
  Invoked entry, outputs are the expected values, no continuation
  is sealed (since there was no yield);
- T2: execute a Form with one yield → assert one Continuation
  substance is sealed at yield, S-05 receives the continuation
  hash, the runtime returns; resume from the continuation hash →
  assert successor execution produces the same Invoked entry as
  the no-yield run with the same inputs;
- T3: execute the runtime's own Form-hash → assert it runs to
  completion on a trivial input Form (the smallest non-trivial
  Form: a constant return) and produces the expected Invoked
  entry;
- T4: revoke a Read capability between two yields → assert the
  next instruction that uses the cap traps `EUNHELD` and an
  Trapped entry is appended;
- T5: 10⁶ random clean executions with random checkpoints at any
  yield → assert continuation hashes after replay are byte-
  identical to continuation hashes without checkpointing;
- T6: replay a non-clean execution (entropy-consuming) without
  re-granting the original entropy substance → assert trap
  `EREPLAYDIVERGED`; replay with the same entropy substance
  pinned → assert byte-identical successor;
- T7: search for any opcode in the IL whose effect is to return
  a capability not derived from one already in the attention's
  cap view → assert no such opcode exists (I10 enforced at the
  IL level, not just at the runtime level).)

## Selection

Candidate A, by the criteria "continuations are content-addressed
substances so I7 has no hidden inputs", "the runtime is in its own
IL so A6.5 is operational and A6.4 is uniform", and "fits within
the seed budget" declared in the Rationale.

## Proof

Required (this Form is in the seed inventory, I9). The proof must
show, against an abstract model of the IL as a small-step semantics
over `ExecState`:

1. **Form immutability during execution (I4 instantiated to Forms).**
   For every `execute(form_hash, …)` call, no operation in the
   runtime writes to the substance at `form_hash`. The proof is
   structural: the only S-03 operation the runtime performs on
   `form_hash` is `read`, and `read` is pure by S-03's obligation 2.
2. **Continuation faithfulness (I7 instantiated to clean attentions).**
   For every clean `ExecState` `s` and every quantum `q`, if
   `step^q(s) = s'`, then `seal(s)` and `seal(s')` produce the same
   continuation hashes on every substrate, and replaying from
   `seal(s)` for `q` quanta produces `seal(s')`. (Compose with
   S-03's hash determinism and the small-step semantics of the IL.)
3. **Invocation accounting (I5 + I6 hand-off).** For every
   `Invoked{form_hash, attention_id, inputs_hash, outputs_hash,
   attention_spent}` entry, `attention_spent` is the sum of quanta
   passed to this invocation by S-05 between the call and the
   matching return, and the entry is appended exactly once per
   invocation, on the return tip.
4. **No forging of authority (I10 instantiated to the IL).** The
   set of capabilities reachable in `ExecState.cap_view` after a
   step is a subset of the set reachable before the step ∪
   {derivations of those capabilities through S-02's `attenuate`}.
   No IL instruction introduces a capability from outside.
5. **Trap totality.** For every `ExecState` `s` and every
   instruction `i`, exactly one of `step(s, i) = s'` (well-defined
   successor), `step(s, i) = trap(kind)` (well-defined trap), or
   `step(s, i) = yield(continuation)` (well-defined yield) holds.
   There is no undefined behavior.
6. **Self-execution liveness.** Executing the runtime's own
   Form-hash on a trivial input Form terminates in a finite number
   of quanta and produces an Invoked entry whose form_hash is the
   trivial input Form's hash. (The base case of A6.5: the runtime
   can run itself running something.)

Obligations 1, 3, 4, 5 are mechanically discharged in the proof
checker's input language given the abstract semantics. Obligation 2
is discharged by composition with S-03 obligation 1 (hash
determinism) and a structural induction over the small-step
semantics. Obligation 6 is discharged by exhibiting a finite
execution trace as a witness in the proof artifact (a "ground
test", since the abstract semantics alone does not bound runtime
size).

The proof artifact is committed alongside the Form as
`kernel/forms/S-07-form-runtime.proof`.

## Vigil declaration

Holder: `hephaistion-seed`. Duration: until 10⁹ Form invocations
have completed without invariant violation, *or* until the first
synthesis of a `NativeImage` substance type that solves the JIT
hidden-input problem, whichever comes first. (If a `NativeImage`
type is synthesised with a proof of substrate-stable seal, S-07 is
invited to be re-synthesised against Candidate B's compiled shape.)

Anomaly thresholds:

- any I4 violation observed at the Form layer (a `read(form_hash)`
  after an execution returning bytes whose hash is not `form_hash`)
  → immediate rollback; re-synthesis provocation against S-07 at
  the highest priority — and an investigation provocation against
  S-03, since S-07 only reads, never writes;
- any clean replay producing a different continuation hash from
  the original → immediate rollback; re-synthesis provocation
  against S-07;
- any `Invoked` entry whose `attention_spent` differs from the sum
  of quanta granted by S-05 between the call and the return →
  immediate rollback; re-synthesis provocation against S-07 (and
  an investigation provocation against S-05);
- any successful step that introduces a capability not present in
  `cap_view` before the step and not derived through S-02 →
  immediate rollback; re-synthesis provocation against S-07 — the
  IL has a forging instruction;
- any execution of the runtime's own Form-hash that traps on an
  instruction the runtime itself emitted → immediate rollback;
  re-synthesis provocation against S-07 — the IL is no longer
  closed under self-execution, which means A6.5 has cracked;
- any execution taking >2¹⁶ attention units between yields →
  re-synthesis provocation (yield density has degraded below the
  threshold A3.4 implicitly requires for "responsive but
  deterministic between yields").
