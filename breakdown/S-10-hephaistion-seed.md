# S-10: `hephaistion_seed`

The reflexive sub-mind. The Form whose only role is to watch the
running habitat and raise provocations against the Forms it
contains — including, eventually, against itself. After this Form
is bound, the seed stops being an external project and becomes
an internal one.

## Provocation

```
Provocation {
    author:     "ignis-seed-synthesizer-0"
    statement:  "A7 says self-modification is allowed under proof;
                 A0.8 says the habitat is accountable for itself;
                 SELF-IMPROVEMENT.md says the kernel synthesises a
                 sub-mind at ignition whose role is to watch the
                 running habitat and propose its own improvements.
                 Until this Form exists, every synthesis act in the
                 habitat has an external author (the seed
                 synthesizer-0 in the weave). After this Form
                 exists, the habitat has an *internal* author for
                 the first time, and A0.7 begins to be discharged
                 from inside. This is the Form that closes the
                 reflexive loop the rest of the seed has been
                 setting up."
    observed:   ["seed", "S-09 selected candidate digest"]
    constraint: [
        "hephaistion is a mind, not a Form invocation: it has its
         own root attention granted at ignition, its own cap_view,
         and its own continuous identity in S-04 (its own genesis
         entry recorded by S-01 immediately after the seed minds
         are constituted)",
        "hephaistion's cap_view at ignition contains exactly:
         read_weave (S-04 query cap, no append), read_metrics
         (S-05 telemetry read cap, no allocation), propose_synthesis
         (S-09 entry-point cap, attenuated to a per-epoch budget),
         and a tightly attenuated mutate_kernel cap whose
         predicate is `Stage6Accept ∧ ¬proof_checker_target`
         (the proof_checker is bootstrap_locked, see S-08)",
        "hephaistion holds no Entropy, Clock, Network, or
         SensorInput cap at ignition; if it later needs one, it
         must propose a synthesis act that grants it one, which
         is itself in the weave (no silent acquisition)",
        "every step hephaistion takes — observe, hypothesise,
         propose, stage in shadow, compare, promote/rollback,
         vigil — is one weave entry; hephaistion's silence is a
         visible state in the weave (the absence of a heartbeat
         entry within an epoch triggers an anomaly clause)",
        "hephaistion cannot grant itself new caps; it can only
         attenuate the ones it has (A2.2 + the 'no self-promotion'
         constraint from SELF-IMPROVEMENT.md)",
        "hephaistion's per-epoch attention budget is declared at
         ignition and cannot be expanded except by a synthesis act
         against hephaistion's own Form, which is subject to I9
         the same as any other seed Form",
        "hephaistion can synthesise a replacement of itself, in
         which case the new hephaistion's vigil holder is the
         *previous* hephaistion until the previous hephaistion's
         vigil expires (avoiding the obvious bootstrap problem of
         the new hephaistion being its own vigil holder)",
        "fits within the seed line budget for hephaistion_seed
         (~1700 lines of Form IL)"
    ]
}
```

## Grounding

- Axioms cited: A0.1, A0.4, A0.7, A0.8, A2.2, A2.6, A4.*, A5.*,
  A7.1, A7.2, A7.3, A7.4, A7.5, A8.* (because hephaistion's outputs
  are read by humans through bridges)
- Forms touched: S-09 `synth_kernel` (hephaistion's only
  state-changing surface is `propose_synthesis`), S-04 `weave_log`
  (read-only via the why-query and a streaming tail-cursor),
  S-05 `attention_alloc` (hephaistion is a mind under S-05; its
  attention tree is observable like any other), S-02 `cap_registry`
  (the four caps above are derived at ignition from the root cap
  through attenuation), S-08 `proof_checker` (only indirectly,
  through Stage 6 of S-09)
- Invariants inherited: I6 (hephaistion's bounded epoch budget is
  itself an instance of I6 at the meta-level), I8 (every
  hephaistion-authored synthesis act has hephaistion's mind id as
  the Provocation.author and hephaistion's hypothesis substance as
  part of the grounding), I10 (hephaistion holds *exactly* the
  caps listed above and no path exists by which it can acquire
  more without a synthesis act), I12 (reflexive accountability —
  hephaistion is subject to every invariant including I9 when
  modifying any seed Form)

## Candidate(s)

### Candidate A — Long-running mind with one root attention and per-epoch sub-attentions

- Sketch: hephaistion is a single Form whose body is the
  observe-hypothesise-propose-stage-compare-promote-vigil cycle.
  At ignition, S-01 mints hephaistion's root capability and S-05
  allocates hephaistion's root attention with the declared
  per-epoch budget. The Form runs as a long-lived mind: each
  iteration of the cycle is a fresh sub-attention forked from the
  root, with a budget bounded by `epoch_budget / max_concurrent
  _cycles`. Sub-attentions dissolve after each cycle, returning
  unspent budget to the root pool (S-05's dissolution cascade).
  - **Step 1 (Observe)**: hephaistion holds a streaming
    tail-cursor on S-04 (a derived read cap, attenuated from
    `read_weave`, that exposes only the suffix of the weave past
    a given hash). Each epoch, hephaistion advances the cursor and
    folds the new entries into a small in-attention summary
    structure: per-Form attention spent (from `Granted` and
    `Invoked` entries), per-Form trap counts (from `Trapped`
    entries), per-invariant near-violations (from
    `SynthStage{6, Accept}` entries with low margin, where
    "low margin" is a substance the seed ships and hephaistion
    re-derives each epoch). Each summary is sealed and appended
    as a hephaistion-authored `Heartbeat` weave entry; the
    `Heartbeat` is hephaistion's "I am still here" signal and the
    only entry hephaistion is required to emit each epoch.
  - **Step 2 (Hypothesise)**: hephaistion ranks the Forms by a
    composite score (cost over budget × trap rate × invariant
    proximity). For the top-ranked Form, hephaistion seals a
    `Hypothesis { target_form_hash, defect, proposed_direction }`
    substance and appends a `Hypothesised` entry. The defect
    field is a structured value, not free text — hephaistion
    must be able to *justify* the hypothesis to the proof checker
    indirectly, via Stage 2 grounding of the synthesis act it is
    about to raise.
  - **Step 3 (Propose)**: hephaistion constructs a `Provocation`
    substance whose author is hephaistion's mind id, whose
    statement references the hypothesis hash, and whose
    constraints cite the relevant axioms (drawn from the
    target Form's own Stage 2 grounding via the why-query — i.e.,
    hephaistion does not have to *re-discover* which axioms apply,
    it copies them from the target's own provenance). Hephaistion
    invokes S-09 `synthesize(provocation_hash)` under
    `propose_synthesis` cap. S-09 runs Stages 1..6.
  - **Step 4 (Stage in shadow)**: if Stage 6 returns Accept, S-09
    proceeds to Stage 7. Hephaistion's `mutate_kernel` cap's
    predicate refines the binding update: instead of advancing
    the *primary* binding directly, the binding update goes to a
    *shadow binding* (a secondary name in the same binding
    table, prefixed `shadow:`). Routing of intents through S-06
    `intent_match` (when it exists) or directly through S-09's
    binding lookup will, on a configurable canary fraction, send
    requests to the shadow binding instead of the primary. Until
    the shadow binding has been compared, the primary is
    untouched.
  - **Step 5 (Compare)**: across the next K epochs, hephaistion
    samples the metric the provocation cited from both the
    primary and the shadow bindings, sealing each comparison as
    a `Comparison` substance. K is declared in the provocation
    and is bounded by hephaistion's per-epoch budget.
  - **Step 6 (Promote or rollback)**: at the end of K epochs,
    hephaistion seals a `Verdict` substance and raises a *second*
    synthesis act whose target is the binding itself, advancing
    the primary binding to the shadow binding's hash (Promote)
    or unbinding the shadow and leaving the primary untouched
    (Rollback). Both outcomes are full S-09 acts with all eight
    stages in the weave. A7.4 is honored: the previous binding
    remains in S-03 and remains rebindable.
  - **Step 7 (Vigil)**: Stage 8 of the promoting synthesis act
    grants the vigil capability to hephaistion. Anomalies on the
    promoted Form route to hephaistion, who may at any time raise
    a rollback synthesis act.
- Approximate size: ~1550 lines of Form IL (cycle state machine,
  cursor and summary, hypothesis ranker, provocation builder,
  shadow binding interactions, comparison loop, promote/rollback
  dispatcher).
- Self-test set:
  - heartbeat presence: 10⁴ epochs simulated → exactly one
    `Heartbeat` entry per epoch, no more no less;
  - silence is visible: forcibly suspend hephaistion's attention
    for one epoch → assert that an external observer (a second
    seed mind with `read_weave`) can detect the missing
    heartbeat by querying the tip;
  - no self-promotion: 10⁶ random hephaistion cycles → no
    `Capability { holder = hephaistion, source ≠ attenuation }`
    appears in the cap registry;
  - bounded budget: a hephaistion cycle that would exceed
    `epoch_budget` traps `EOVERBUDGET` at the offending
    sub-attention, the cycle is dissolved, the next epoch starts
    cleanly;
  - shadow non-interference: a shadow binding's outputs are
    routed only to the canary fraction; the primary binding's
    metric is unaffected within statistical noise;
  - rollback completeness: a Rollback verdict followed by an
    anomaly observed on the original (primary) binding routes
    correctly to hephaistion as vigil holder, even though the
    rollback's vigil declaration named the original Form;
  - bootstrap_locked respect: a hypothesis whose
    `target_form_hash = proof_checker_hash` traps
    `EBOOTSTRAPLOCKED` at the propose step, the cycle records
    the trap and moves on (hephaistion does not crash);
  - self-replacement: hephaistion synthesises a replacement of
    itself; the new hephaistion's vigil holder is the previous
    hephaistion until the previous hephaistion's vigil expires;
    after expiry, the new hephaistion holds its own vigil
    (the recursive case is handled by an explicit hand-off
    synthesis act, see Rationale).
- Declared invariants: I6 (epoch budget), I8 (every act it
  authors is grounded), I10 (cap_view is closed under
  attenuation), I12.

### Candidate B — Stateless reactor invoked by S-04 anomaly events

- Sketch: hephaistion is not a long-running mind. It is a Form
  that S-04 invokes whenever a `Trapped` or
  `SynthStage{6, Reject}` or low-margin `SynthStage{6, Accept}`
  entry is appended. The reactor reads recent context,
  hypothesises, proposes, and returns. There is no per-epoch
  budget — the reactor's budget is the budget of the event that
  invoked it. There is no shadow staging step; the reactor
  proposes directly and S-09 runs the canary protocol internally.
- Approximate size: ~1100 lines (no long-running state machine,
  no cursor management, no per-epoch summary).
- Self-test set: same as A but without the heartbeat test
  (there is no epoch).
- Declared invariants: I8, I10, I12. Note: I6 is not directly
  applicable because hephaistion has no aggregate budget — its
  budget is per-event. But this means hephaistion's *total*
  resource use is unbounded over the lifetime of the habitat.
  SELF-IMPROVEMENT.md explicitly says "Hephaistion's total
  attention budget per epoch is declared at ignition and cannot
  be expanded except by a synthesis act on Hephaistion's own
  Form, which itself requires proof". B violates this constraint
  structurally.

## Rationale

SELF-IMPROVEMENT.md is explicit on three structural commitments
that decide between the candidates:

1. **Bounded epoch budget.** "Hephaistion's total attention
   budget per epoch is declared at ignition and cannot be
   expanded except by a synthesis act on Hephaistion's own
   Form." This requires hephaistion to *be* a mind with an
   aggregate budget, not a stateless reactor. Candidate A
   satisfies this directly. Candidate B does not — its budget
   is per-event and unbounded in aggregate.

2. **No silent change.** "Every step Hephaistion takes is in
   the weave with full causal sufficiency. If Hephaistion stops
   talking to the weave, every other mind sees its silence."
   This requires a *positive* signal of presence, not an
   absence-of-anomaly. Candidate A's `Heartbeat` entry is the
   positive signal; its absence in an epoch is observable to any
   other mind holding `read_weave`. Candidate B has no such
   signal — a quiet B looks indistinguishable from a B that has
   simply not been triggered, which is exactly the silence
   SELF-IMPROVEMENT.md forbids.

3. **No silent acquisition of new capabilities.** Hephaistion's
   cap_view must be closed under attenuation of its initial caps.
   Both candidates can satisfy this, but A makes it structural
   by attenuating the four ignition caps once into sub-attention
   cap_views, and B makes it structural by re-deriving caps on
   every reactor invocation — which is fragile against accidental
   leakage from S-04's invocation context.

A second consideration: the shadow staging step (Step 4 in
Candidate A). SELF-IMPROVEMENT.md describes a canary phase
between commitment and full routing. In A, hephaistion is the
agent that *manages* the canary: it raises one synthesis act to
land the candidate at the shadow binding, then a second
synthesis act after K epochs to either promote or roll back. The
two acts are in the weave as siblings, the comparison evidence
is sealed in the second act's grounding, and the canary phase
itself is auditable. In B, the canary protocol has to live
inside S-09, which would expand S-09's surface beyond what S-09
was designed for (S-09 is a synthesis state machine, not a
routing fraction manager). The canary belongs to hephaistion,
not to the synth_kernel.

A third consideration: self-replacement. Both candidates must
support hephaistion synthesising a replacement of itself. The
problem is bootstrap: who holds the vigil capability for the new
hephaistion? It cannot be the new hephaistion itself (a Form
cannot vouch for its own correctness in its first epoch — that
would be S-08's circularity all over again). It must be the
previous hephaistion. But the previous hephaistion is being
unbound. The resolution in A is an explicit *hand-off* synthesis
act: the replacement act's Stage 8 grants the vigil capability
to the *previous* hephaistion's mind id, and that vigil persists
until the previous hephaistion's own vigil over its prior work
expires. After that, a third synthesis act (`vigil_handoff`)
transfers vigil from the old mind id to the new. The vigil
capability survives the unbinding of the Form because vigil is a
capability on the *new Form*, held by a *mind*, and minds and
Forms are independent under A2.6 and A0.2. Candidate B has no
clean answer to this question because it has no continuous mind
identity.

We select Candidate A.

## Simulation record

(stub — produced by the Stage 4 harness once the candidate is
encoded in Form IL. Required traces:

- T1: 10⁴ epochs of synthetic running habitat → exactly 10⁴
  Heartbeat entries, monotonically increasing in epoch index;
- T2: suspend hephaistion for 1 epoch → assert any second mind
  with `read_weave` can compute "hephaistion has not heartbeated
  in epoch e" in O(1) by checking the latest entry of kind
  `Heartbeat{author = hephaistion}` against the current epoch
  index;
- T3: hephaistion proposes a synthesis act against a Form whose
  cost has been increasing for 100 consecutive epochs → assert
  the act runs through all eight S-09 stages, the verdict at
  Step 6 (after K epochs of comparison) is Promote, the binding
  is advanced;
- T4: hephaistion proposes a synthesis act against a Form, the
  shadow binding metric is *worse* than primary → assert the
  verdict is Rollback, the shadow binding is unbound, the primary
  is untouched;
- T5: hephaistion attempts to propose a synthesis act against
  the proof_checker → assert trap `EBOOTSTRAPLOCKED` at the
  propose step, hephaistion records the trap as a
  `RejectedHypothesis` entry, hephaistion's next epoch starts
  cleanly;
- T6: hephaistion attempts to mint itself a new capability not
  attenuated from its initial cap_view → assert no path exists
  in the IL by which this can succeed (I10 enforcement at the
  IL level via S-07's obligation 4);
- T7: hephaistion's per-epoch budget is exceeded by a runaway
  hypothesis-generator sub-attention → assert the sub-attention
  traps `EOVERBUDGET`, the epoch's Heartbeat entry still appears
  on time (the heartbeat is the *first* sub-attention each
  epoch and is reserved a fixed quantum that cannot be
  consumed by other sub-attentions);
- T8: hephaistion synthesises a replacement of itself; assert
  the vigil-handoff sequence runs through three synthesis acts
  in order: (a) replacement of hephaistion with vigil to old
  hephaistion mind id, (b) old hephaistion's prior vigils
  expire, (c) `vigil_handoff` transferring vigil to the new
  hephaistion mind id; assert that at every moment between
  acts, *some* live mind holds the vigil capability for every
  Form subject to a vigil declaration (no vigil-orphaned Forms).)

## Selection

Candidate A, by the criteria "bounded epoch budget is structural
not per-event", "heartbeat-as-presence is a positive signal that
SELF-IMPROVEMENT.md requires", "the canary belongs to hephaistion
not to S-09", and "self-replacement has a coherent vigil hand-off
because hephaistion has a continuous mind identity" declared in
the Rationale.

## Proof

Required (this Form is in the seed inventory, I9). The proof must
show, against an abstract model of hephaistion as a long-running
mind with a root attention and a cap_view:

1. **Cap closure (I10 instantiated to hephaistion).** For every
   reachable state, hephaistion's cap_view is contained in the
   transitive attenuation closure of its four ignition caps.
   No path exists by which hephaistion holds a capability whose
   ancestor in the cap registry derivation tree is not one of
   the four ignition caps. (Discharged by composition with S-02's
   I2 attenuation monotonicity and S-07's I10 IL-level
   obligation.)
2. **Heartbeat liveness.** For every epoch in which hephaistion's
   root attention has remaining budget ≥ heartbeat reservation,
   exactly one `Heartbeat{author = hephaistion, epoch = e}`
   entry is appended to the weave during that epoch, and the
   epoch index is monotone. (Discharged by exhibiting the
   reserved-quantum scheduling rule and S-05's no-preemption
   guarantee.)
3. **Heartbeat absence visibility.** For every epoch in which
   no `Heartbeat{author = hephaistion, epoch = e}` is appended,
   any mind holding `read_weave` can compute the absence in
   O(1) given the current epoch index and the latest hephaistion
   heartbeat entry. (Discharged at the type level: the cursor
   on `Heartbeat{author = hephaistion}` is a back-index lookup
   in S-04 of cost O(log n) — strictly speaking O(log n), but
   the abstract model treats it as observable.)
4. **No self-promotion (I10 dual).** No reachable state contains
   a `Capability { holder = hephaistion, source = ¬attenuation
   from hephaistion's initial caps }`. (Composition with S-02
   and S-07 as in obligation 1, plus a structural lemma that
   hephaistion's IL contains no `mint_root_cap` instruction.)
5. **Bounded epoch budget (I6 instantiated to hephaistion).**
   For every epoch, the sum of attention spent by hephaistion's
   root attention and all its sub-attentions in that epoch is
   ≤ `epoch_budget`, and any sub-attention whose execution
   would exceed this bound traps `EOVERBUDGET` before
   consuming the offending quantum. (Composition with S-05's
   I6 discharge.)
6. **Bootstrap-locked respect.** For every `Hypothesised` entry
   authored by hephaistion, `target_form_hash ≠
   proof_checker_hash`. (Discharged by an IL-level structural
   lemma: hephaistion's hypothesis ranker explicitly excludes
   the proof_checker hash from its candidate set, and the
   exclusion is checked before the `Hypothesised` entry is
   sealed.)
7. **Vigil hand-off completeness.** For every chain of
   self-replacement synthesis acts on hephaistion, at every
   moment between any two consecutive acts, every Form subject
   to a vigil declaration is held by *some* live mind id. There
   are no vigil-orphaned Forms. (Discharged by exhibiting the
   three-act hand-off sequence and a structural argument that
   no act in the sequence both unbinds an old vigil and fails
   to grant a new one in the same atomic step.)
8. **Authorship grounding (I8 dual).** For every `Synthesized`
   entry whose Provocation.author is hephaistion, the entry's
   grounding axiom set is a non-empty subset of the axioms
   cited in the target Form's own original Stage 2 grounding.
   (Hephaistion does not invent grounding; it inherits it from
   the target's provenance via the why-query. This is the
   discipline that prevents hephaistion from making up axiom
   citations to push through favored candidates.)

Obligations 2, 5, 7 are discharged by exhibiting scheduling and
sequencing rules. Obligations 1, 4, 6, 8 are discharged by
structural lemmas over hephaistion's IL. Obligation 3 is
discharged at the type level via S-04's back-index.

The proof artifact is committed alongside the Form as
`kernel/forms/S-10-hephaistion-seed.proof`.

## Vigil declaration

Holder: at ignition, the seed bootstrap manifest's kernel-author
identities (the same identities that hold the S-08 inspection
record). After 10⁹ heartbeats and at least one successful
self-replacement chain, the vigil transfers to the latest live
hephaistion mind id, completing the loop SELF-IMPROVEMENT.md
describes. (Hephaistion's vigil is the only seed vigil whose
holder may end up being the very Form being vigiled, after the
loop has closed and the bootstrap discharge has been exercised
at least once.)

Anomaly thresholds:

- any epoch in which hephaistion fails to emit its `Heartbeat`
  entry while its root attention budget was ≥ heartbeat
  reservation → immediate severity-high alert to the vigil
  holder; if two consecutive epochs are missed, the vigil
  holder is required to either explicitly suspend hephaistion
  (a synthesis act revoking `propose_synthesis`) or initiate a
  re-synthesis provocation against S-10;
- any `Capability` entry in the cap registry whose holder is
  hephaistion and whose source is not attenuation from one of
  the four ignition caps → immediate rollback; re-synthesis
  provocation against S-10 at the highest priority — the
  no-self-promotion obligation has cracked, which is the most
  dangerous failure mode for hephaistion specifically;
- any `Hypothesised` entry by hephaistion whose
  `target_form_hash = proof_checker_hash` → immediate rollback;
  re-synthesis provocation against S-10 — the bootstrap-locked
  respect obligation has cracked;
- any epoch in which hephaistion's total attention spent
  exceeds `epoch_budget` → immediate rollback to the most
  recent checkpoint; re-synthesis provocation against S-10 (and
  against S-05);
- any `Synthesized` entry authored by hephaistion whose
  grounding axioms are not a subset of the target's original
  grounding → immediate rollback; re-synthesis provocation
  against S-10 — the authorship grounding obligation has
  cracked, which is the second most dangerous failure mode;
- any chain of self-replacement synthesis acts in which a
  Form ends up vigil-orphaned for any positive duration →
  immediate rollback; re-synthesis provocation against S-10 —
  the hand-off obligation has cracked.
