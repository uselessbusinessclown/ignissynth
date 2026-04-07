# S-11: `bridge_proto`

The Form that lets a human reach the habitat without becoming an
inhabitant of it. Bridges are minds, not interfaces. This Form
specifies what *being a bridge* means at the seed level: the
typed-conversation reference bridge, expressed as a Form whose
ignition produces a single bridge mind whose cap_view is whatever
the bootstrap manifest happens to grant it.

## Provocation

```
Provocation {
    author:     "ignis-seed-synthesizer-0"
    statement:  "A8.1 says a bridge is itself a mind in the
                 habitat. A8.2 says a bridge translates and does
                 not privilege. A8.3 says a human acting through a
                 bridge has exactly the capabilities the bridge
                 attenuates and grants for the duration of the
                 interaction. A8.5 says every act a human takes
                 through a bridge is recorded in the weave with
                 the same fidelity as any inhabitant act. Until
                 this Form exists, the habitat is sealed: minds
                 form intents, intents are matched, fulfillers
                 run, but no signal from outside the habitat
                 reaches inside it and no signal from inside
                 reaches a human. The seed needs at least one
                 bridge so that its first act of self-improvement
                 (Hephaistion proposing a synthesis) can be
                 *witnessed* by humans through the receipt
                 mechanism A8.5 promises. Without this Form, the
                 seed is honest only in principle."
    observed:   ["seed", "S-06 selected candidate digest"]
    constraint: [
        "the bridge is a mind: at ignition it is granted a root
         attention through S-05 and a cap_view through S-02
         attenuated from the seed bootstrap caps; from inside the
         habitat, the bridge is indistinguishable from any other
         mind",
        "the bridge offers exactly two surfaces to its human
         counterparty: typed_intent_in (a typed value the human
         supplies, after the bridge's translation step) and
         typed_receipt_out (a typed value the bridge emits after
         the corresponding match has produced a verdict)",
        "the bridge translates between human modalities and typed
         intents; at the seed, the only modality is text-shaped
         typed conversation, so the translation step is a parse
         from a small typed surface language into an Intent
         substance — there is no NLP, no LLM call, no free-form
         goal field; A4.2 forbids it",
        "the bridge's cap_view is exactly what the bootstrap
         manifest grants it and what subsequent synthesis acts
         attenuate from those caps; the bridge cannot grant the
         human counterparty any capability the bridge does not
         hold and is not authorized to delegate (A8.2)",
        "every typed_intent_in is sealed as an Intent substance
         and matched through S-06 under the bridge's policy cap;
         the bridge does not bypass S-06; the bridge does not
         have a privileged shortcut to any fulfiller",
        "every typed_receipt_out is a sealed Receipt substance
         whose contents reference the original Intent hash, the
         Matched entry hash, the fulfiller's verdict against the
         intent's acceptance form, and the human-facing rendering
         of the result; the receipt is appended as a `BridgeOut`
         weave entry by the bridge mind, so A8.5 is structural",
        "human silence is *not* a state the bridge tracks; the
         bridge is request-driven, and between requests it is
         simply a mind with idle attention; if a human walks
         away mid-interaction, no follow-up exists",
        "fits within the seed line budget for bridge_proto
         (~1400 lines of Form IL)"
    ]
}
```

## Grounding

- Axioms cited: A0.1, A0.4, A2.2, A2.6, A4.1, A4.2, A4.5, A8.1,
  A8.2, A8.3, A8.4, A8.5
- Forms touched: S-02 `cap_registry` (the bridge's cap_view is
  attenuated from the bootstrap caps), S-05 `attention_alloc`
  (the bridge has a root attention like any mind), S-06
  `intent_match` (every bridge act flows through the matcher),
  S-04 `weave_log` (the bridge appends `BridgeIn`, `Matched`
  by way of S-06, and `BridgeOut` entries), S-03
  `substance_store` (Intent and Receipt substances), S-08
  `proof_checker` (only indirectly, via Stage 6 of any synthesis
  the bridge raises against itself)
- Invariants inherited: I1 (the bridge holds a policy cap to use
  S-06 like any other mind), I5 (BridgeIn and BridgeOut are
  weave entries), I8 (the bridge's identity is a `Synthesized`
  entry at ignition; subsequent self-modifications are full S-09
  acts), I10 (the bridge holds *exactly* the caps the bootstrap
  manifest gave it and their attenuations), I11 (bridge
  subordination — capabilities held by the bridge are bounded by
  what the kernel granted at startup, and the bridge cannot
  acquire authority by virtue of the human on its other side)

## Candidate(s)

### Candidate A — Single bridge mind, request-driven, parse-only translator

- Sketch: `bridge_proto` is a Form whose ignition produces a
  single bridge mind. The bridge mind's body is:
  1. **Listen.** The bridge holds an `inbound_endpoint` capability
     — a sealed substance of type `Endpoint` that carries a
     deserialiser from the typed surface language to a `BridgeRequest
     { goal_value, inputs[], constraints[], budget,
     acceptance_form_hash, human_id_token }` value. The endpoint
     is the *only* path from outside the habitat to the bridge,
     and it accepts only well-formed values in the surface
     language. (At seed time, the surface language is a single
     small grammar; later bridges may synthesise larger grammars
     under their own synthesis acts.)
  2. **Parse and seal.** On receiving a request, the bridge
     parses it into typed values, checks well-formedness against
     the surface language schema, and seals an `Intent`
     substance through S-03. The Intent's `acceptance_form_hash`
     comes from the request, but the bridge verifies that the
     acceptance form is a substance the bridge already holds
     (via its cap_view) — a request whose acceptance form
     references a hash the bridge cannot read traps `EUNHELDFORM`,
     and the bridge emits a Receipt with verdict
     `Rejected{EUNHELDFORM}` rather than entering the matcher.
     The bridge appends a `BridgeIn{intent_hash, human_id_token,
     endpoint_hash}` weave entry.
  3. **Match.** The bridge calls S-06 `match(intent_hash)` under
     its policy capability. The policy cap's predicate is
     bounded — it admits only fulfillers the bridge is
     authorized to delegate to, which is exactly the
     attenuation rule of A8.2. The bridge does not synthesise
     a new policy cap mid-conversation; the policy cap is part
     of the bridge's static cap_view, and changing it requires
     a synthesis act against the bridge's own Form.
  4. **Hand off.** The bridge takes the Matched entry's
     fulfiller hash and sub-attention id, and either invokes
     the fulfiller through S-07 (Form fulfillers) or forwards
     the intent to another mind (mind fulfillers). The bridge
     itself does not run the fulfiller — A4.3 forbids it from
     bypassing the matcher's separation of "what to do" from
     "doing it".
  5. **Verdict.** When the fulfiller's execution returns, the
     bridge takes the result, checks it against the intent's
     `acceptance_form_hash` (this is the explicit acceptance
     check A4.5 promises), and seals a `Receipt {intent_hash,
     matched_entry_hash, verdict, human_facing_rendering,
     fulfiller_attention_spent}` substance. Verdicts are
     `Accepted{result_hash}`, `Rejected{reason_hash}`, or
     `Indeterminate{reason_hash}` — there is no implicit
     success.
  6. **Emit.** The bridge appends a `BridgeOut{receipt_hash,
     intent_hash, human_id_token}` weave entry and writes the
     receipt to the inbound endpoint's reply channel. The
     human reads the receipt; the bridge's cycle ends.
  7. **Idle.** Between requests, the bridge is a mind with idle
     attention. It does not poll. It does not maintain
     conversational state outside what is sealed in the
     receipts that have been emitted. Any continuity across
     requests must be reconstructed from the weave by the next
     request, the same way any other mind reconstructs context.
- Approximate size: ~1300 lines of Form IL (parser + endpoint
  layout + the request/match/handoff/verdict/emit cycle +
  cap_view bookkeeping).
- Self-test set:
  - well-formed request → BridgeIn entry, Matched entry,
    fulfiller invocation, Verdict, Receipt, BridgeOut entry,
    in that order;
  - malformed request (fails the surface grammar) → Receipt with
    `verdict = Rejected{EILLFORMED}`, no BridgeIn entry, no
    Matched entry (the request never became an Intent
    substance);
  - request whose acceptance_form_hash is not held by the bridge
    → BridgeIn entry, Receipt with `verdict =
    Rejected{EUNHELDFORM}`, no Matched entry;
  - request matched against a fulfiller that returns a result
    failing the acceptance form → BridgeIn, Matched, Verdict
    `Rejected{acceptance_failed}`, BridgeOut;
  - the bridge attempts to grant the human a capability the
    bridge does not itself hold → trap at the cap operation
    (S-02 enforces I2 attenuation monotonicity), no Matched
    entry, Receipt with verdict `Rejected{cap_attenuation_
    violation}`;
  - the bridge attempts to acquire a capability not derived
    from its initial cap_view → no IL path exists (S-07
    obligation 4 enforces this);
  - 10⁴ random requests with random failure injection → for
    every BridgeOut entry, the corresponding BridgeIn entry
    exists and references the same `human_id_token`, and no
    BridgeOut precedes its BridgeIn in the weave;
  - human silence: a request that is sent and then no follow-up
    arrives for 10⁶ epochs → the bridge's idle attention
    accumulates no state, no `Heartbeat` is required (the
    bridge is not Hephaistion), and a subsequent request from
    the same `human_id_token` is treated as a fresh
    interaction with no implicit context;
  - reproducibility from the weave: a human auditing their own
    receipts can, by walking the BridgeIn → Matched → BridgeOut
    chain by hash, reconstruct exactly which fulfiller ran on
    their behalf, with which intent, against which acceptance
    form, with which verdict, on which attention budget — A8.5
    is operational.
- Declared invariants: I1, I5, I8, I10, I11.

### Candidate B — Per-conversation bridge instances spawned by a meta-bridge factory

- Sketch: instead of a single long-running bridge mind, a meta-
  bridge factory spawns a fresh bridge mind per conversation.
  Each conversation-bridge has its own attention root and its
  own cap_view, and dissolves at the end of the conversation.
  The motivation is per-conversation isolation: two humans
  cannot, even in principle, observe each other's intents
  through the bridge's shared state because the bridges are
  distinct minds.
- Approximate size: ~1100 lines for the meta-factory + ~600 for
  the conversation-bridge body = ~1700 lines, over budget.
- Self-test set: same as A, plus an isolation test: two
  concurrent conversations produce two distinct bridge minds
  whose `read_weave` cap_views are scoped to their own
  BridgeIn/BridgeOut chains.
- Declared invariants: I1, I5, I8, I10, I11. Note: the
  meta-bridge factory has the authority to *mint* new bridge
  minds, which means the meta-bridge holds a capability that is
  strictly more powerful than any individual bridge — it is a
  capability over the *creation of minds*, which the seed has
  not previously synthesised. This is a new kind of power
  introduced for an isolation property the seed has no use
  for yet.

## Rationale

A8.1 says a bridge is itself a mind, and A8.4 says the habitat is
not obliged to provide a desktop. The synthesis question is
whether the bridge is *one* mind or *one per conversation*.

Candidate A makes the bridge one mind. The bridge's discipline
is that it carries no conversational state across requests —
continuity is reconstructed from the weave on demand, the same
way any other mind reconstructs context. This matches A0.2 (a
mind is a continuous causal process whose state is the
consequence of its prior states and inputs) by treating "the
weave entries the bridge has authored" as the bridge's persistent
state and treating "anything else" as not state at all.

Candidate B makes per-conversation bridges. The motivation is
isolation, but the seed already has isolation: each request
produces a distinct Intent substance with a distinct
`human_id_token`, and the matcher's per-intent sub-attention
boundary is enforced by S-05. Two concurrent requests through the
*same* bridge mind cannot observe each other's intents because
their fulfiller sub-attentions are siblings under the bridge's
root attention with no shared cap view. The isolation B promises
is already provided by S-05's attention tree and S-02's cap
attenuation; B is solving a problem the rest of the seed has
already solved.

Worse, B introduces a *new* kind of capability — the authority to
mint new bridge minds — which is strictly more powerful than any
individual bridge's authority. This meta-capability has no clear
attenuation rule, no clear vigil holder, and no analogue elsewhere
in the seed. It is the kind of structural addition the seed
discipline forbids: do not synthesise capacities the seed has no
provocation for. Provocations come from observation; we have no
observation that one-bridge-many-conversations is inadequate.

A second consideration: A8.2 (bridges translate, they do not
privilege) and A8.3 (humans never have ambient authority). In A,
the bridge's cap_view is static and is exactly what the bootstrap
manifest granted, so the human's effective authority is exactly
what the bridge attenuates per request, and the attenuation is
visible in the BridgeIn entry's policy cap id. In B, each
conversation-bridge has its own cap_view, and the question of
what *that* cap_view contains is answered by the meta-bridge —
which means the meta-bridge is, transitively, the authority over
all human authority in the habitat. That centralisation is the
opposite of what A8.3 wants.

A third consideration: the receipt chain (A8.5). In A, BridgeIn
→ Matched → BridgeOut is a contiguous walk in the weave by hash,
and any human auditing their own receipts can reproduce the chain
by walking back. In B, the conversation-bridge's identity is in
the chain too, and the chain has a "which bridge mind handled
this conversation" step that does not exist in A. The B chain is
not harder to audit but it is *longer*, and length here means
"more state to verify" without buying anything A4.5 or A8.5 ask
for.

A fourth consideration: the seed's first-hour discipline. The
seed should ship with the smallest bridge that satisfies A8 and
no more. A is exactly that: one mind, one cap_view, request-
driven, parse-only translator, no NLP, no LLM, no free-form goal
field. Larger bridges (graphical, voice, multimodal) are
synthesis problems for the running habitat, raised by Hephaistion
or by a human-through-a-bridge, with their own provocations and
their own proofs. The seed bridge does not have to be the only
bridge; it has to be the *first* bridge.

We select Candidate A.

## Simulation record

(stub — produced by the Stage 4 harness once the candidate is
encoded in Form IL. Required traces:

- T1: well-formed request → assert BridgeIn entry on the tip,
  followed by Matched entry, followed by Invoked entry from the
  fulfiller, followed by BridgeOut entry; receipt's verdict is
  `Accepted` and references the fulfiller's output hash;
- T2: malformed surface-language request → assert no BridgeIn
  entry (the request never became an Intent), Receipt emitted
  with `verdict = Rejected{EILLFORMED}`, written to the
  endpoint's reply channel;
- T3: request with acceptance_form_hash referencing a substance
  the bridge cannot read → BridgeIn entry, Receipt with
  `Rejected{EUNHELDFORM}`, no Matched entry;
- T4: request matched, fulfiller runs, fulfiller's output fails
  the acceptance form check → BridgeIn, Matched, Invoked,
  Verdict `Rejected{acceptance_failed}`, BridgeOut; the fulfiller
  was *invoked* but the verdict is failure;
- T5: 10⁴ random request sequences from random `human_id_token`
  values → for every BridgeOut entry, the matching BridgeIn
  entry exists, and the (BridgeIn, Matched, BridgeOut) triple
  is contiguous on the tip modulo other minds' interleaved
  appends;
- T6: search the bridge's cap_view at any reachable state →
  every cap is reachable from the bootstrap manifest's caps by
  attenuation; no cap has any other source;
- T7: search for any IL path by which the bridge grants the
  human a cap not attenuated from one the bridge holds → no
  such path exists (S-07 obligation 4);
- T8: human silence: send one request, do not send a follow-up
  for 10⁶ S-05 ticks → the bridge's idle attention accumulates
  no per-conversation state, no entries are appended by the
  bridge during the silence; a subsequent request from the same
  token is processed as a fresh BridgeIn with no implicit
  context;
- T9: receipt chain reconstruction: pick any BridgeOut entry
  authored by the bridge, walk back along its references; assert
  the walk recovers exactly one BridgeIn entry, one Matched
  entry, one Invoked entry, one Receipt substance, and the
  intent substance, in finite time bounded by the chain
  length.)

## Selection

Candidate A, by the criteria "the seed already has the isolation
B promises through S-02 + S-05", "B introduces a meta-capability
to mint minds with no provocation for it", "A8.3 is honored by a
static cap_view that does not centralise human authority into a
meta-bridge", and "the receipt chain is shortest and most
auditable in A" declared in the Rationale.

## Proof

Required (this Form is in the seed inventory, I9). The proof
must show, against an abstract model of the bridge as a mind
with a static cap_view and a request-driven cycle:

1. **Bridge subordination (I11).** For every reachable state,
   the bridge's cap_view is contained in the transitive
   attenuation closure of the caps granted by the bootstrap
   manifest. There is no path by which the bridge acquires a
   capability whose ancestor is not in that set. (Composition
   with S-02's I2 and S-07's I10 IL-level obligation; identical
   in shape to S-10 obligation 1.)
2. **No human privilege (A8.3 / I11 dual).** For every Matched
   entry whose calling mind is the bridge, the policy cap used
   by the matcher is held by the bridge itself, not by the
   human counterparty. The human's `human_id_token` appears in
   the BridgeIn entry's metadata but never in the cap_view of
   any attention. (Discharged by an IL structural lemma:
   the bridge has no instruction that takes a `human_id_token`
   as an argument to a cap-derivation operation.)
3. **Receipt chain completeness (A8.5).** For every BridgeOut
   entry authored by the bridge with `(receipt_hash, intent
   _hash, human_id_token)`, there exists exactly one BridgeIn
   entry by the same bridge with the same `human_id_token` and
   the same `intent_hash`, appended earlier on the tip; and for
   every successful match, exactly one Matched entry with the
   same `intent_hash` exists between the BridgeIn and the
   BridgeOut. (Discharged by structural induction over the
   bridge's cycle: each cycle iteration emits exactly the
   triple in order or terminates early with a Rejected
   verdict, in which case the Matched entry is absent and the
   verdict explicitly cites a pre-match failure reason.)
4. **Acceptance check totality (A4.5 instantiated to bridge
   verdicts).** For every Receipt substance authored by the
   bridge with `verdict = Accepted{result_hash}`, the result
   substance was checked against the intent's
   `acceptance_form_hash` and the check returned true. For
   every Receipt with `verdict = Rejected{reason}` or
   `verdict = Indeterminate{reason}`, the check either was
   not run (because the failure occurred before match) or was
   run and returned false. There is no Receipt whose verdict
   was emitted without one of these two paths having been
   taken. (Mechanically discharged.)
5. **No silent state (A8.5 dual).** The bridge holds no state
   across requests other than substances reachable by hash from
   weave entries it has authored. There is no IL register, no
   in-memory map, no cache that survives between requests
   without being sealed. (Discharged by an IL structural lemma:
   the bridge's per-cycle local state is dropped at cycle
   end, and the only state that survives is what `seal` and
   `append` made into substances and weave entries.)
6. **Human silence as no-op.** For every interval between two
   requests from the same `human_id_token`, the bridge appends
   zero weave entries with that token. (The bridge does not
   poll, does not heartbeat per conversation, does not
   reconnect.)

Obligations 1, 2, 3, 4, 5 are mechanically discharged in the
proof checker's input language. Obligation 6 is discharged
trivially by the cycle's request-driven shape (no timer
instruction in the IL).

The proof artifact is committed alongside the Form as
`kernel/forms/S-11-bridge-proto.proof`.

## Vigil declaration

Holder: at ignition, the seed bootstrap manifest's kernel-author
identities (the same identities that hold the S-08 inspection
record and the initial S-10 vigil). After 10⁹ BridgeIn entries
have been processed without invariant violation, the vigil
transfers to `hephaistion-seed`, completing the loop in which
the habitat takes responsibility for its own boundary with
humans.

Anomaly thresholds:

- any BridgeOut entry whose receipt's verdict is `Accepted` but
  whose result substance fails the acceptance form check on
  replay → immediate rollback; re-synthesis provocation against
  S-11 at the highest priority — the bridge has lied to a
  human, which is the most serious failure mode the bridge has
  by virtue of A8.5;
- any Matched entry whose calling mind is the bridge and whose
  policy cap was held by anything other than the bridge itself
  → immediate rollback; re-synthesis provocation against S-11
  (and S-02) — A8.3 has cracked, the human has been granted
  ambient authority;
- any cap in the bridge's cap_view at any state whose ancestor
  in the cap derivation tree is not in the bootstrap manifest
  → immediate rollback; re-synthesis provocation against S-11
  — I11 has cracked;
- any state-changing operation by the bridge that survives a
  cycle boundary as IL register state rather than as a sealed
  substance → re-synthesis provocation against S-11 — A8.5's
  silent-state prohibition has cracked;
- any BridgeOut entry without a matching prior BridgeIn entry
  with the same `human_id_token` and `intent_hash` → immediate
  rollback; re-synthesis provocation against S-11 — the
  receipt chain has cracked, which means a human is reading a
  receipt for an act that has no recorded prompt;
- any successful synthesis act raised by the bridge against
  itself whose Stage 6 proof did not pass S-08's standard
  discharge → immediate rollback; re-synthesis provocation
  against S-11 (and S-09 and S-08).
