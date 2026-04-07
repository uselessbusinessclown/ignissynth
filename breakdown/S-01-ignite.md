# S-01: `ignite`

The singular Form that mints authority from the axioms themselves.
After it runs once, it ceases to exist as a callable Form (I10).

## Provocation

```
Provocation {
    author:     "ignis-seed-synthesizer-0"
    statement:  "Before any mind can hold anything, the habitat must
                 contain at least one capability. By I10, no Form may
                 return a capability the caller did not already hold.
                 Therefore exactly one Form must exist whose effect is
                 to bring the first capability into being from the
                 axioms — and that Form must, as its final act,
                 unmake itself so that the exception it embodies cannot
                 be reused. Without `ignite`, the habitat is a sealed
                 jar with nothing inside it."
    observed:   ["seed"]
    constraint: [
        "executes exactly once in the lifetime of a habitat instance",
        "after execution, no Form in the running system has the
         signature of `ignite` and no weave entry can replay it",
        "produces a single root capability whose rights are the
         universal set, whose budget is the total declared energy of
         the substrate, and whose holder is the seed mind",
        "every subsequent capability in the habitat is a transitive
         attenuation of this root",
        "ignition is itself a synthesis act recorded in the weave with
         grounding back to A0.7, A2.1, I10",
        "fits within the seed line budget for ignite (≤ 400 lines of
         Form IL, justified by being the smallest Form in the seed)"
    ]
}
```

## Grounding

- Axioms cited: A0.3, A0.4, A0.7, A2.1, A2.6
- Forms touched: none (this is seed-original, and is the *first* Form
  by definition — nothing else can exist in the running system before
  it)
- Invariants inherited / discharged: I1, I8, I10 (this Form is the
  named exception to I10, and the burden of the exception is that
  ignition must be unrepeatable)

## Candidate(s)

### Candidate A — Self-erasing entry point

- Sketch: `ignite` is encoded as a Form whose entry point performs
  three indivisible steps inside a single weave entry:
  1. mint the root capability `R` with universal rights, full
     substrate budget, holder = seed mind id;
  2. write a `Synthesized` weave entry `E0` whose grounding cites
     A0.7, A2.1, I10, whose rationale substance is this very
     breakdown's hash, and whose effect is "root cap minted";
  3. overwrite its own Form table slot with the sealed substance
     `IGNITED` (a zero-arity Form whose only effect is to fail with
     `EIGNITED`), so that any subsequent attempt to call `ignite`
     traps and is recorded as an anomaly.
- Approximate size: ~250 lines of Form IL.
- Self-test set:
  - first call returns a capability `R` such that `rights(R) =
    UNIVERSAL` and `budget(R) = substrate_budget`;
  - second call traps with `EIGNITED` and produces a weave entry of
    type `IgnitionReplayAttempted`;
  - the weave entry written by the first call hashes to the value
    cited by the seed mind's first checkpoint;
  - on cold restart from a checkpoint taken after `ignite` ran, the
    Form table slot for `ignite` is `IGNITED`, not the original.
- Declared invariants: I1, I8, I10 (with stated exception).

### Candidate B — Axiom-keyed one-shot oracle

- Sketch: `ignite` is not a Form in the Form table at all. Instead, the
  seed loader treats the BLAKE3 hash of the concatenation of the axiom
  files (`axioms/A0..A8`) as a *one-shot key*. At first boot, the
  loader uses this key to seal the root capability and emit `E0`; it
  then writes the key into the substance store as `BURNED`, so any
  later attempt by the loader to repeat the procedure observes the
  `BURNED` mark and refuses. There is never an addressable `ignite`
  Form, so the I10 exception is enforced by the *absence* of the Form
  rather than by self-erasure.
- Approximate size: ~120 lines, but ~80 of them belong to the seed
  loader rather than to the seed Form table — moving the budget
  outside the synthesis surface.
- Self-test set: same as A, plus:
  - tampering with the axiom files between checkpoint and restart
    changes the key and the loader refuses to ignite at all
    (a feature, not a bug — re-grounds A0.7).
- Declared invariants: I1, I8, I10.

## Rationale

A0.7 says new structure enters only through the synthesis protocol.
A2.1 says capabilities are unforgeable except at ignition. I10 says
no Form returns ungranted authority *except* `ignite`, and `ignite`
does not exist as a callable Form after ignition. The synthesis
question is therefore: where does the named exception live, and how
is its single use enforced?

Candidate A localises the exception inside the Form table itself. The
exception is small, named, and its disappearance is observable in the
weave (`E0` and the `IGNITED` slot). Any later mind that examines the
Form table sees the scar and can verify it.

Candidate B pushes the exception out of the Form table and into the
loader. This is structurally cleaner — the I10 exception literally
does not exist in the running system because the Form does not exist
in the running system — but it relocates trust from the Form layer
into a loader layer that is *not itself* a synthesised artifact. That
re-introduces the very property A0.7 forbids: a piece of the habitat
that is not a theorem of the axioms but a precondition of them. The
loader becomes a second source of normativity, parallel to the
axioms, with no synthesis chain of its own.

We select Candidate A. The cost is that the seed Form table contains,
forever, a slot whose history is "once was `ignite`, now is
`IGNITED`". The benefit is that this scar is a *first-class artifact*
of the habitat, visible to every later inspection, and is itself
covered by the synthesis protocol. The exception named by I10 is
honored in exactly one place, and that place is inside the system it
governs.

A secondary consideration: Hephaistion (A7, S-10) is required to be
able to reason about every Form in the seed. If `ignite` lives inside
the loader, Hephaistion cannot reason about its provenance without a
second reasoning surface for "loader things". Candidate A keeps
Hephaistion's surface uniform.

## Simulation record

(stub — produced by the Stage 4 harness once the candidate is encoded
in Form IL. Required traces:

- T1: cold boot → first call → assert `rights(R) = UNIVERSAL`,
  `budget(R) = substrate_budget`, weave length = 1, weave[0].type =
  `Synthesized`, weave[0].grounding ⊇ {A0.7, A2.1, I10};
- T2: cold boot → first call → second call → assert second call
  traps `EIGNITED`, weave length = 2, weave[1].type =
  `IgnitionReplayAttempted`;
- T3: cold boot → first call → checkpoint → restart → call → assert
  trap `EIGNITED`, weave[1] preserved across restart;
- T4: 10⁶ random sequences of (call, checkpoint, restart, call) →
  assert exactly one successful ignition per habitat instance.)

## Selection

Candidate A, by the criteria "the I10 exception is honored in exactly
one place and that place is inside the synthesis surface" and
"Hephaistion has a uniform reasoning surface over the seed" declared
in the Rationale.

## Proof

Required (this Form is in the seed inventory, I9). The proof must
show, against an abstract model of the Form table and the weave:

1. **Uniqueness of ignition.** For any execution trace, the number of
   weave entries of type `Synthesized` whose effect is "root cap
   minted" is at most one.
2. **Universality of the root.** The capability `R` returned by the
   single successful ignition has `rights(R) = UNIVERSAL`,
   `budget(R) = substrate_budget`, and is held by the seed mind id.
3. **Closure of authority.** For every capability `C` in any
   reachable state, there exists a chain of attenuations from `R` to
   `C`. (This is the I10 closure lemma; `ignite` is the only Form
   whose proof discharges the base case of this induction.)
4. **Self-erasure.** After the successful ignition, no transition in
   the abstract model leads to a state in which the Form table slot
   for `ignite` is anything other than `IGNITED`.
5. **Replay refusal.** Any call to the slot named `ignite` after the
   successful ignition produces a weave entry of type
   `IgnitionReplayAttempted` and returns `EIGNITED`.

Obligations 1, 4, and 5 are mechanically discharged in the proof
checker's input language. Obligation 2 is discharged by direct
definition (the substance bytes that encode `ignite` are
content-addressed and inspectable). Obligation 3 is the induction
hypothesis used by every later seed Form's proof; `ignite` discharges
only its base case.

The proof artifact is committed alongside the Form as
`kernel/forms/S-01-ignite.proof`.

## Vigil declaration

Holder: `hephaistion-seed`. Duration: for the entire lifetime of any
habitat instance derived from this seed (this Form has no
re-synthesis horizon — it runs once and then is, by construction,
immutable scar tissue). Anomaly thresholds:

- any weave entry of type `IgnitionReplayAttempted` → log at the
  highest severity, but do not trigger re-synthesis (a replay attempt
  is the *expected* failure mode, not an anomaly in `ignite` itself);
- any reachable state in which the Form table slot for `ignite` is
  not `IGNITED` after `E0` exists → immediate rollback to the last
  checkpoint preceding `E0`, and a re-synthesis provocation against
  S-01 itself (the seed has been corrupted);
- any capability `C` for which no attenuation chain to `R` exists →
  immediate rollback and a re-synthesis provocation against the
  Form responsible for minting `C` (not against `ignite`, which has
  already discharged its base case).
