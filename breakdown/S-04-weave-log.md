# S-04: `weave_log`

The causal record. Every act in the habitat lands here as a single
entry whose hash references its causes. Without the weave, A5 is a
slogan and I5, I7, I8, I9 have no enforcement surface.

## Provocation

```
Provocation {
    author:     "ignis-seed-synthesizer-0"
    statement:  "A5.1 says every act is recorded; A5.2 says entries
                 are linked by hash; A5.3 says contents are private
                 but structure is public; A5.4 says 'why does this
                 exist?' is always answerable; A5.5 says the weave
                 outlives the minds. Until this Form exists, no act
                 can be recorded, which means by I5 no act can be
                 *valid*: there is nothing for later state to
                 reference. The substance store (S-03) gives us a
                 floor for matter; the weave gives us a spine for
                 cause."
    observed:   ["seed", "S-03 selected candidate digest"]
    constraint: [
        "append(entry) is total: returns the new entry's hash, links
         it to the previous tip, and is observable in a single
         atomic step",
        "every entry is itself a substance in S-03 (so I4 covers it)",
        "the tip hash is part of every checkpoint and uniquely
         identifies the entire causal history",
        "queries are by hash only (A5.3): no enumeration of entries
         by holder, by type, or by time",
        "the why-query (A5.4) returns the transitive set of entries
         whose outputs the queried substance hash transitively
         depends on, in finite time bounded by the size of that set",
        "no entry can be amended after append; reordering is
         structurally impossible (the previous-hash link is in the
         entry's own hash)",
        "fits within the seed line budget for weave_log
         (~1700 lines of Form IL)"
    ]
}
```

## Grounding

- Axioms cited: A0.4, A0.8, A1.1, A1.4, A5.1, A5.2, A5.3, A5.4, A5.5
- Forms touched: S-03 `substance_store` (every weave entry is sealed
  through S-03; the weave's tip hash is itself a substance)
- Invariants inherited: I4 (entries are substances), I5 (causal
  completeness), I8 (synthesis grounding — every Synthesized entry
  has non-empty rationale), I10 (no enumeration of entries except by
  hash already held)

## Candidate(s)

### Candidate A — Strict Merkle chain with per-substance back-index

- Sketch: the weave is a strict chain: each entry is a sealed
  substance `Entry { prev_tip, kind, inputs[], outputs[], grounding,
  rationale_hash, attention_spent }`. The tip is one hash. Append
  takes the current tip, builds the new entry referencing it, seals
  it through S-03, and atomically advances the tip pointer (the tip
  pointer itself is a substance, re-sealed on every append, whose
  hash is the only mutable thing in the system — and even it is
  mutable only in the trivial sense that one substance hash replaces
  another). For A5.4, a per-substance back-index maps each output
  hash to the entry hash that produced it; the index is itself a
  persistent hash trie (same shape as S-03's). The why-query starts
  at the queried substance, looks up the producing entry, then
  recursively visits the entries that produced that entry's inputs.
- Approximate size: ~1500 lines of Form IL (entry layout +
  append-and-tip-advance + back-index + why-traversal).
- Self-test set:
  - append → new tip's prev = old tip;
  - any attempt to seal an entry whose prev ≠ current tip ⇒ trap
    `ESTALE` at the Form level (not at the store level — the store
    accepts any sealable substance);
  - back-index lookup for an output hash returns exactly one entry
    hash;
  - why-query for a substance never produced returns the empty set;
  - why-query for a substance produced from N entries returns a set
    of size ≤ N and the set is closed under "producer-of-input";
  - 10⁶ random (append, why) ops with no holder may enumerate the
    weave by anything other than a hash they were given;
  - replay from any checkpoint produces a tip hash equal to the
    original.
- Declared invariants: I4, I5, I8, I10.

### Candidate B — Per-mind sub-weaves merged at synchronization points

- Sketch: each mind appends to its own private chain; periodically
  (or on cap-mediated cross-mind interaction) the chains are merged
  into a global Merkle DAG. Cheaper for parallel append by many
  minds, since per-mind appends do not contend on a single tip.
- Approximate size: ~1300 lines for per-mind chains + ~600 lines for
  the merge protocol = ~1900 lines, slightly over budget.
- Self-test set: same as A, plus a merge invariant: after merge, the
  set of entries reachable from the global tip equals the union of
  the entries reachable from each per-mind tip just before merge,
  with no duplicates.
- Declared invariants: I4, I5, I8, I10. Note: I5 ("for every
  state-changing event there is *exactly one* entry") becomes
  fragile under concurrent merge — the proof obligation has to argue
  that merge is associative and commutative on the entry multiset,
  which is non-trivial.

## Rationale

A5.2 says entries are linked by hash; A5.1 says every act produces
exactly one entry. The simplest reading is a strict total order: one
tip, one append at a time, one new entry. Candidate A is exactly
that. Its tip is a single hash, its history is a chain (the
degenerate case of a Merkle DAG), and its why-query is a recursive
walk of the back-index. Every property A5 names is structural.

Candidate B trades that simplicity for parallel append throughput.
The seed has neither workload data nor concurrency yet (S-05
`attention_alloc` does not yet exist), so the throughput argument is
speculative. Worse, the merge protocol introduces a state in which
"the weave" is temporarily ambiguous: between two merges, there is
no single tip whose hash means "the entire causal history". I5
demands exactly-one-entry-per-event; under merge, this is true only
*after* merge, and the period between merges contains state changes
whose canonical entry depends on which merge runs next. That is an
abstraction the seed cannot afford to ship with — Hephaistion can
re-synthesise toward parallel append once attention scheduling
exists and the seed has a workload to justify the cost.

A5.3 (privacy by hash) is automatic in both candidates: entries
reference input and output substances by hash, and reading those
substances requires holding their capabilities through the cap
registry (S-02). The weave is fully public in structure and the
substance store enforces privacy in content.

A5.4 (queryable why) is direct in A via the back-index. In B it
requires walking each per-mind chain plus the merge records, which
is finite but more involved.

The decisive consideration is the same as in S-03: at every
checkpoint, the tip hash must be the *single* identity of "the
entire causal history of the habitat at this checkpoint". Candidate
A gives this for free. Candidate B does not.

We select Candidate A.

## Simulation record

(stub — produced by the Stage 4 harness once the candidate is
encoded in Form IL. Required traces:

- T1: append(e₁) → append(e₂) → assert tip(e₂).prev = hash(e₁) and
  tip(e₂) ≠ tip(e₁);
- T2: attempt to append an entry whose prev ≠ current tip → assert
  trap `ESTALE`, weave unchanged;
- T3: produce substance s through entry e → why(s) = {e} ∪ why of
  each input of e, and the result is a finite set;
- T4: 10⁴ (append, checkpoint, restart, append) sequences → final
  tip hash equals tip hash of the un-checkpointed run with the same
  appends;
- T5: search for any operation whose return type is an enumeration
  of entries not derivable from a hash supplied by the caller →
  assert no such Form exists in the exported surface (I10 absence
  test, same shape as S-03's T5);
- T6: append entry whose grounding is empty → assert trap
  `EUNGROUNDED` for entries of kind `Synthesized` (I8 enforcement at
  the weave layer);
- T7: 10⁶ random append ops → assert that for every output hash, the
  back-index points to exactly one entry hash.)

## Selection

Candidate A, by the criteria "the tip hash is the single identity of
the entire causal history at a checkpoint" and "I5 holds without a
merge proof obligation" declared in the Rationale.

## Proof

Required (this Form is in the seed inventory, I9). The proof must
show, against an abstract model of the weave as a sequence of
sealed entries with a back-index:

1. **Tip uniqueness.** At every reachable state there is exactly one
   tip hash, and `tip.prev` is the previous tip hash or `⊥` for the
   first entry (the genesis entry, which is the `Synthesized` entry
   for `ignite` itself, S-01).
2. **Append totality.** For every well-formed entry e with `e.prev =
   current_tip`, `append(e)` advances the tip to `hash(e)` and
   inserts the producer-of-each-output mappings into the back-index;
   for every entry whose `prev` is not the current tip, `append(e)`
   has no effect on the weave and traps `ESTALE`.
3. **No amendment.** There exists no operation in the exported
   surface whose effect is to remove or replace an entry already
   referenced by any later entry's prev or by the back-index.
   (Equivalently: the only mutable thing in the weave's
   exported surface is the tip pointer, and even it can only
   advance — i.e., be replaced by an entry whose prev is the
   pointer's current value.)
4. **Why-query soundness and completeness.** For every substance
   hash s, `why(s)` returns the set of entries E such that there is
   a chain `e₀ → e₁ → … → eₖ` in the back-index from the entry
   producing s, where each step is "producer of one of this entry's
   inputs". The set is finite (bounded by the number of entries
   transitively reachable) and contains every such entry (no entry
   that contributed to s is missing).
5. **No enumeration.** There exists no operation in the exported
   surface whose return type is an unbounded `Set<EntryHash>` or
   `Set<SubstanceHash>` not function-of-arguments. (I10 instantiated
   to causality; same type-level discharge as S-03 obligation 6.)
6. **Grounding obligation for synthesised entries.** For every
   appended entry of kind `Synthesized`, `entry.grounding` is
   non-empty (cites at least one axiom by id) and
   `entry.rationale_hash` references a non-empty substance in S-03.
   (I8 enforced at the weave layer rather than at the synthesis
   layer, so that no Form bypassing the synthesis Form can still
   land an ungrounded entry.)

Obligations 1–4 and 6 are mechanically discharged in the proof
checker's input language. Obligation 5 is discharged by type-level
inspection of the exported surface, identical in shape to S-03's
obligation 6.

The proof artifact is committed alongside the Form as
`kernel/forms/S-04-weave-log.proof`.

## Vigil declaration

Holder: `hephaistion-seed`. Duration: until 10⁹ append operations
have completed without invariant violation. Anomaly thresholds:

- any successful append whose prev ≠ the current tip at append time
  → immediate rollback to the most recent checkpoint; re-synthesis
  provocation against S-04 at the highest priority — the spine has
  bent;
- any why-query that returns a substance hash unreachable in the
  back-index from the queried hash → immediate rollback;
  re-synthesis provocation against S-04;
- any successful append of a `Synthesized` entry with empty
  grounding or empty rationale → immediate rollback; re-synthesis
  provocation against the *appender* (not against S-04, which has
  already discharged obligation 6 if its surface refused the entry,
  or against S-04 if the surface accepted it);
- any append operation taking >2048 attention units → re-synthesis
  provocation (the constant factor of the chain has become
  unaffordable; Hephaistion is invited to propose a parallel-append
  replacement, at which point the merge proof obligation referred
  to in Candidate B must be discharged);
- discovery of any Form in the running system whose return type
  contains an unbounded `Set<EntryHash>` not derivable from
  arguments → re-synthesis provocation against *that* Form, not
  against S-04.
