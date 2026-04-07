# S-09: `synth_kernel`

The Form that performs synthesis acts. The eight stages of
`synthesis/PROTOCOL.md` are not a document the habitat consults —
they are this Form's executable shape. Every commit to the habitat,
including replacements of seed Forms, flows through here.

## Provocation

```
Provocation {
    author:     "ignis-seed-synthesizer-0"
    statement:  "A0.7 says new structure enters only through the
                 synthesis protocol; A7.1 says self-modification is
                 itself a synthesis act; A7.5 says the habitat is
                 not above its own law. Until this Form exists, the
                 protocol is text and the only synthesizer is an
                 external AI reasoning into the seed at ignition.
                 After this Form exists, the habitat synthesizes
                 from inside itself, on its own attention budget,
                 under its own weave, against its own proof checker.
                 This is the Form that promotes the synthesis
                 protocol from a constraint on humans-with-keyboards
                 to a constraint on the running system."
    observed:   ["seed",
                 "S-04 selected candidate digest",
                 "S-07 selected candidate digest",
                 "S-08 selected candidate digest"]
    constraint: [
        "synthesize(provocation_hash) → SynthResult is the only
         entry point; the eight stages are sequential and each
         stage's substance is sealed before the next begins",
        "every stage produces exactly one weave entry; a synthesis
         act is therefore observable as a contiguous run of eight
         entries on the tip (or fewer if the act fails early — the
         failure stage is the last entry)",
        "the synth_kernel is itself a mind, with its own attention
         tree under S-05; its meta-budget is accounted (A0.8) and
         its own quantum cost per stage is recorded in the trial
         record",
        "Stage 6 (Proof) calls S-08 directly; an Accept advances
         to Stage 7, a Reject terminates the act with a failure
         entry that nonetheless preserves all earlier stage
         substances (A0.4 — the failed attempt is part of the
         history)",
        "Stage 7 (Commitment) updates a binding (name → hash) under
         a capability that the synth_kernel holds and that S-02 has
         registered as the kernel mutation cap; without that cap
         in scope, commitment traps `EUNAUTHORISED`",
        "self-application: the synth_kernel can take itself as the
         Form being replaced (S-09 → S-09'), in which case Stage 6
         passes through S-08's K-of-N consensus discharge for any
         touch on S-08, and through S-08's standard discharge for
         the rest of S-09",
        "fits within the seed line budget for synth_kernel
         (~2000 lines of Form IL)"
    ]
}
```

## Grounding

- Axioms cited: A0.4, A0.7, A0.8, A6.2, A6.5, A7.1, A7.2, A7.3,
  A7.4, A7.5
- Forms touched: S-04 `weave_log` (one entry per stage), S-07
  `form_runtime` (the synth_kernel itself runs under S-07; Stage 4
  simulation runs candidates under S-07 inside a sealed
  sub-attention), S-08 `proof_checker` (Stage 6), S-05
  `attention_alloc` (the synth_kernel is a mind, its stages are
  attentions), S-02 `cap_registry` (Stage 7 binding cap), S-03
  `substance_store` (every stage's record is a substance)
- Invariants inherited: I5 (every stage is a weave entry), I6 (the
  synth_kernel's own meta-budget conserves), I8 (every Synthesized
  entry has grounding — the synth_kernel itself enforces this on
  Stage 2's output), I9 (Stage 6 *is* I9's enforcement at the
  protocol level, complementing S-08's enforcement at the
  proof-checking level), I10 (no commitment without the binding
  cap)

## Candidate(s)

### Candidate A — Sequential state machine, one stage per attention split

- Sketch: the synth_kernel is a Form whose body is a state machine
  over the eight protocol stages. On `synthesize(provocation_hash)`:
  1. Allocate a fresh attention `A_synth` as a child of the
     calling mind's attention, with a budget declared in the
     provocation's `meta_budget` field. `A_synth` is the synthesis
     act's "author mind".
  2. **Stage 1 (Provocation)**: read the provocation substance
     through S-03; verify `author` is non-empty and resolves to a
     mind id known to S-02; seal a `Stage1Record { provocation_hash,
     verified_author }`; append weave entry of kind
     `SynthStage{1, record_hash}`. Trap `EANONYMOUS` if the author
     check fails.
  3. **Stage 2 (Grounding)**: read the provocation's `constraint`
     list; for each constraint, identify the axiom ids it references
     (the seed ships an axiom-id index as a sealed substance, see
     below); verify every constraint cites at least one axiom id
     present in the index; collect the touched Form hashes the
     constraint declares; for each touched Form, fetch its declared
     invariants from the Form-substance's manifest; seal a
     `Stage2Record`; append `SynthStage{2}`. Trap `EUNGROUNDED` if
     any constraint cites no axiom or any touched Form's invariants
     cannot be fetched.
  4. **Stage 3 (Candidates)**: invoke the candidate-generation
     sub-attention. The sub-attention is itself a Form-execution
     under S-07, with its own budget cap from `A_synth`'s remaining
     pool. The sub-attention is given the grounding substance and
     produces zero or more candidate substances of type
     `Form` together with rationale substances. The synth_kernel
     does not generate candidates itself — it *invokes* a generator
     Form whose hash is named in the provocation. (For seed time,
     the generator hash is the trivial generator that reads the
     candidates verbatim from the provocation's `seed_candidates`
     field — at seed time, candidates are produced by the external
     synthesizer and handed in.) Trap `ENOCANDIDATES` if zero
     candidates are produced.
  5. **Stage 4 (Simulation)**: for each candidate, allocate a fresh
     sealed sub-attention with no outward capabilities (no S-02
     reachable from inside, no S-04 append, no entropy). Inside the
     sub-attention, run the candidate against its self-test set,
     the inherited invariant predicates, a fuzzing harness derived
     from the candidate's input schema, and the regression cases
     drawn from the weave by the why-query (S-04). The sub-attention
     produces a `TrialRecord` substance. The synth_kernel collects
     all trial records, seals them as a `Stage4Record`, and
     appends `SynthStage{4}`. Trap `EALLFAILED` if every candidate
     failed its self-test set.
  6. **Stage 5 (Selection)**: for each surviving candidate, evaluate
     the selection criteria from the provocation. Selection is
     itself a synthesis act in miniature: it has a rationale
     (sealed) explaining why this candidate over the others. The
     losing candidates are *not unpinned* — they remain pinned by
     the Stage 5 record substance, satisfying A0.4 and the
     protocol's discard prohibition. Seal `Stage5Record { winner,
     rationale, losers }`; append `SynthStage{5}`.
  7. **Stage 6 (Proof)**: if the winner replaces a Form declared
     `proof_required` (i.e., any seed Form), invoke S-08:
     `check(proof_hash, claim_hash)` where the claim is built from
     the winner's `form_hash_after`, the replaced Form's
     `form_hash_before`, and the union of declared invariants. On
     Reject, append `SynthStage{6, Reject{reason}}` and terminate
     the act. On Accept, append `SynthStage{6, Accept}`. For
     replacement of S-08 itself, the Accept case requires K-of-N
     co-signatures per S-08's bootstrap discharge — the
     synth_kernel checks the co-signature count and traps
     `EBOOTSTRAPSHORT` otherwise.
  8. **Stage 7 (Commitment)**: under the kernel mutation
     capability, atomically advance the binding (name → hash) from
     `form_hash_before` to `form_hash_after`; append a `Synthesized`
     entry into the weave whose grounding cites the Stage 2 axiom
     ids and whose rationale_hash is the Stage 5 rationale. (This
     is the *one* entry whose kind is `Synthesized`; the seven
     `SynthStage{i}` entries above are stage records, not
     synthesis claims. The Stage 7 `Synthesized` entry references
     the seven preceding stage records by hash, so a why-query for
     the new Form returns the entire eight-stage history.) Trap
     `EUNAUTHORISED` if the binding cap is not in scope.
  9. **Stage 8 (Vigil)**: seal a `VigilDeclaration` substance from
     the candidate's declared vigil; mint a vigil capability through
     S-02 and grant it to the holder named in the declaration;
     append `SynthStage{8}`. The vigil is now live and any anomaly
     observed by S-05, S-04, or S-08 that names the new Form's
     hash routes to the vigil holder.
  Throughout: the synth_kernel's own meta-budget per stage is
  recorded in each stage's record substance, so the cost of
  synthesising any Form is itself a queryable property of the
  weave.
- Approximate size: ~1850 lines of Form IL (state machine + stage
  records + sub-attention orchestration + axiom-id index loader).
- Self-test set:
  - happy-path: a trivial provocation with a no-op candidate
    succeeds in eight stages, producing eight stage entries plus
    one Synthesized entry; the binding is updated;
  - anonymous-author trap: a provocation with empty author traps
    `EANONYMOUS` at Stage 1 and produces exactly one stage entry
    (the failure entry);
  - ungrounded trap: a provocation whose constraints cite no
    axioms traps `EUNGROUNDED` at Stage 2 with two stage entries;
  - all-failed trap: a provocation whose every candidate fails
    its self-test set traps `EALLFAILED` at Stage 4 with four
    stage entries (provocation, grounding, candidates, the
    failed trials);
  - proof-rejected: a provocation whose winner cannot be proved
    appends a Stage 6 Reject entry and terminates with six stage
    entries; the previous binding is unchanged;
  - losing candidates pinned: after a successful synthesis with
    three candidates, the two losers are still readable through
    S-03 (their hashes are reachable from the Stage 5 record);
  - meta-budget conservation: a synthesis whose stages collectively
    consume more than `A_synth.budget_at_creation` traps
    `EOVERBUDGET` at the offending stage and rolls back the act
    (no Stage 7 emission, no binding update);
  - self-application: synthesising a replacement of S-09 itself
    runs through all eight stages with Stage 6 routed through
    S-08, and the new S-09's hash becomes the bound `synth_kernel`
    only after Stage 7.
- Declared invariants: I5, I6, I8, I9, I10.

### Candidate B — Stage-as-coroutine, single attention with persistent stage state

- Sketch: instead of allocating a fresh sub-attention per stage,
  the synth_kernel is a single coroutine that runs all eight stages
  in one attention, persisting stage state across yields. The
  motivation is to avoid the overhead of attention allocation and
  cap-derivation per stage. Stages communicate through
  in-coroutine variables rather than through sealed substances.
- Approximate size: ~1500 lines of Form IL (~350 fewer than A
  because no per-stage attention or per-stage record substance).
- Self-test set: same as A, but without the per-stage entries —
  Candidate B emits one `SynthAct` entry per completed synthesis
  rather than eight per-stage entries.
- Declared invariants: I5 (degraded — see Rationale), I6, I8, I9,
  I10.

## Rationale

The protocol document says "every stage produces exactly one
weave entry" only implicitly, in the form of "a synthesis act is
observable in the weave". The synthesis question is whether
"observable" means "by hash of the act as a whole" or "by hash of
each stage individually".

Candidate A makes each stage a first-class weave entry with its
own substance. The benefit is enormous and easy to under-rate:
because the seven `SynthStage{i}` entries are sealed substances
referenced by hash from the final `Synthesized` entry, the
eight-stage history of any seed Form is recoverable by a single
why-query against its hash. Hephaistion (S-10), reasoning about a
Form, can read not just "how this Form came to exist" but "what
the synth_kernel was thinking at each stage of its arrival,
including the meta-budget it spent, including the candidates it
rejected and why". This is the property A0.4 ("causality is total
and inspectable") cashes out as for synthesis acts specifically.
It is also what makes the protocol document's failure cases
("recorded as such and rejected") *operational* — a failed
synthesis act has the same shape as a successful one, just
truncated, and is queryable identically.

Candidate B compresses an act into one entry. Cheaper, but the
internal state of stages is no longer in the weave — it's in
coroutine variables that are not sealed substances. A failed Stage
4 in B leaves no trial record substance behind unless the
coroutine explicitly emits one; in A, the trial record exists
because Stage 4 cannot complete without sealing it. B's
"observability" is opt-in; A's is structural.

A second consideration: Stage 4 (simulation) requires a sealed
sub-environment with no outward capabilities. In A, this is a
sub-attention with its cap_view restricted at allocation time, and
S-05's no-preemption invariant gives Stage 4 a clean budget
boundary. In B, the simulation runs *inside* the same coroutine
that holds the synth_kernel's caps, and the cap restriction must
be enforced by discipline rather than by structure. That is the
kind of slack I10 forbids: ambient authority leaking from the
synth_kernel into the simulation harness because they share an
attention.

A third consideration: self-application. The synth_kernel must be
able to synthesise a replacement of itself (A6.5, A7.1). In A,
the replacement runs as a fresh `A_synth` under S-05 with its own
budget, and the eight stages of *replacing the synth_kernel* are
themselves recorded as eight stage entries. In B, the replacement
coroutine and the original coroutine share the same attention,
which is incoherent: at the moment of Stage 7 binding update, the
"running" coroutine is the *old* synth_kernel, which is being
unbound. The transition is structurally awkward in B and
structurally trivial in A.

A fourth consideration: I8 (synthesis grounding). A enforces I8 at
Stage 2 by *trapping* if any constraint cites no axiom — the trap
is a stage entry, the failure is in the weave, and the binding is
not advanced. B can enforce the same check, but the failure leaves
no Stage 2 record substance because there is no per-stage record
in B; the grounding failure is observable only as the coroutine's
exit code. That weakens the I8 enforcement surface from
"structural" to "by-convention".

We select Candidate A. The size cost (~350 lines) buys the
structural enforcement of I5, I8, and the no-ambient-authority
property at Stage 4. The seed cannot afford to discharge those
invariants by-convention.

## Simulation record

(stub — produced by the Stage 4 harness once the candidate is
encoded in Form IL. Required traces:

- T1: happy-path synthesis of a no-op Form replacement → assert
  eight `SynthStage{i}` entries followed by one `Synthesized`
  entry, binding updated, vigil cap minted;
- T2: anonymous-author provocation → assert one entry, kind
  `SynthStage{1, Reject{EANONYMOUS}}`, binding unchanged;
- T3: ungrounded provocation → assert two entries, kind
  `SynthStage{2, Reject{EUNGROUNDED}}`, binding unchanged;
- T4: all-candidates-fail provocation → assert four entries
  (Stage 1, 2, 3, 4-with-failures), binding unchanged;
- T5: proof-rejected provocation → assert six entries (Stage 1..6,
  Stage 6 carrying S-08's Reject reason), binding unchanged;
- T6: 10⁴ random provocations with random failure injection at
  random stages → assert that for every act, the count of stage
  entries on the tip equals the index of the failing stage (or 8
  for success), and the binding state matches "advanced iff
  Stage 7 entry exists";
- T7: losing-candidates persistence → after a happy-path with
  three candidates, the two losers are reachable through S-03 by
  hashes named in the Stage 5 record, even after 10⁶ subsequent
  S-03 ops (no reclamation);
- T8: meta-budget overrun → a synth_kernel run whose Stage 4
  simulation budget exceeds the act's meta_budget traps
  `EOVERBUDGET` and produces a Stage 4 entry with `verdict =
  Aborted{EOVERBUDGET}`, binding unchanged;
- T9: self-application → synthesis act whose
  `form_hash_before = current_synth_kernel_hash` and
  `form_hash_after = candidate_synth_kernel_hash` runs through
  all eight stages, Stage 7 advances the `synth_kernel` binding,
  the next call to `synthesize()` is dispatched to the new Form;
- T10: search for any code path by which Stage 7 advances a
  binding without a preceding Stage 6 Accept (for `proof_required`
  Forms) → assert no such path exists.)

## Selection

Candidate A, by the criteria "I5 and I8 are enforced structurally
not by-convention", "Stage 4's no-ambient-authority property is
structural via sub-attention cap restriction", and "self-
application is coherent because old and new synth_kernel run in
distinct attentions" declared in the Rationale.

## Proof

Required (this Form is in the seed inventory, I9). The proof must
show, against an abstract model of the synth_kernel as a state
machine over eight stages with sealed records:

1. **Stage uniqueness.** For every synthesis act with id `a`, the
   weave contains at most one entry of kind `SynthStage{i, …}` for
   each `i ∈ {1..8}` whose record's `act_id = a`. (No stage runs
   twice in the same act.)
2. **Stage ordering.** For every synthesis act `a`, the weave
   indices of `SynthStage{i, …}` entries with `act_id = a` are
   strictly increasing in `i`. (Stages do not run out of order.)
3. **Failure truncation.** For every synthesis act `a` that ends
   in `SynthStage{i, Reject{…}}` for some `i < 8`, no entry
   `SynthStage{j, …}` with `j > i` and `act_id = a` exists in
   the weave, and no `Synthesized` entry references `a`'s stage
   records. (Failed acts do not advance.)
4. **Binding gating.** For every `Synthesized` entry whose
   `form_hash_before` is `proof_required`, the weave contains a
   `SynthStage{6, Accept}` entry with the same `act_id` whose
   record references an S-08 Accept verdict. The proof discharges
   this by composition with S-08's Accept-monotonicity (S-08
   obligation: Accept never spontaneously becomes Reject).
5. **Capability gating (I10).** For every `Synthesized` entry,
   the synth_kernel's `A_synth` attention held the kernel
   mutation capability at the moment of Stage 7 append.
   Discharged by composition with S-02's lookup soundness.
6. **Loser preservation.** For every `Stage5Record { winner,
   losers }` substance reachable from a `SynthStage{5}` entry,
   the substances named in `losers` remain pinned for at least
   the lifetime of the `Synthesized` entry referencing the
   Stage 5 record. (Discharged by the rule that the Stage 5
   record itself pins the loser hashes.)
7. **Meta-budget conservation (I6 instantiated to synth_kernel).**
   The sum of attention spent across the eight stages of any act
   `a` is ≤ `meta_budget(a)`, and any stage whose execution would
   exceed this bound traps `EOVERBUDGET` before sealing its
   record. (Composition with S-05's I6 discharge.)
8. **Self-application liveness.** A synthesis act whose
   `form_hash_before = synth_kernel_hash` terminates in finite
   stages and, on success, advances the `synth_kernel` binding;
   the next `synthesize()` call dispatches through the new
   binding. (Discharged by exhibiting a finite ground execution
   trace as a witness in the proof artifact, identical in
   discharge shape to S-07 obligation 6.)

Obligations 1, 2, 3, 6, 7 are mechanically discharged in the proof
checker's input language. Obligations 4 and 5 are discharged by
composition with S-08 and S-02 respectively. Obligation 8 is
discharged by ground witness.

The proof artifact is committed alongside the Form as
`kernel/forms/S-09-synth-kernel.proof`.

## Vigil declaration

Holder: `hephaistion-seed`. Duration: until 10⁹ synthesis acts have
completed (success or failure) without invariant violation, *or*
until the first successful self-application (a synthesis act
replacing S-09 itself), whichever comes second. (The vigil does
not end at the first self-application — the seed insists that the
*replacement* of the synth_kernel is also vigil-covered for the
same duration as the original, by the new synth_kernel's vigil,
which is granted to Hephaistion at Stage 8 of the replacing act.)

Anomaly thresholds:

- any `Synthesized` entry whose preceding stage entries do not
  form a contiguous run from Stage 1 to Stage 7 → immediate
  rollback; re-synthesis provocation against S-09 at the highest
  priority — the protocol has been bypassed;
- any `Synthesized` entry whose `form_hash_before` is
  `proof_required` and which is not preceded by a
  `SynthStage{6, Accept}` → immediate rollback; re-synthesis
  provocation against S-09 (and an investigation provocation
  against S-08 — one of the two has cracked);
- any successful Stage 7 binding advance whose synth_kernel
  attention did not hold the kernel mutation cap → immediate
  rollback; re-synthesis provocation against S-09 (and against
  S-02);
- any Stage 5 record whose loser hashes become unreachable through
  S-03 before the referencing `Synthesized` entry is itself
  reclaimed → re-synthesis provocation against S-09 — the
  protocol's discard prohibition has cracked;
- any synthesis act whose stage budgets sum to more than the
  declared meta_budget but which nonetheless reached Stage 7 →
  immediate rollback; re-synthesis provocation against S-09 (and
  against S-05);
- a successful self-application of S-09 whose new synth_kernel
  fails its own vigil within the first 10⁶ synthesis acts → the
  rollback path is the previous synth_kernel binding, which by
  A7.4 remains in the substance store and remains rebindable;
  this is the canonical exercise of A7.4 in the seed.
