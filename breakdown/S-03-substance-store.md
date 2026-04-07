# S-03: `substance_store`

The Form that holds the matter of the habitat. Every other Form in
the seed eventually rests on it: the weave, the cap registry's
predicate substances, the Form table itself, the proofs.

## Provocation

```
Provocation {
    author:     "ignis-seed-synthesizer-0"
    statement:  "A1 says the matter of the habitat is typed,
                 content-addressed, sealed substance — no bytes, no
                 pages, no files. Until this Form exists there is
                 nowhere for any substance to live, which means there
                 is nowhere for the weave to record causes, nowhere
                 for predicates to be stored, nowhere for proofs to
                 be checked, nowhere for Forms themselves to reside.
                 The store is the floor of the habitat. If the floor
                 leaks, A1.3 (sealing is permanent), A1.4 (sharing
                 by reference), A1.6 (persistence by pinning) and
                 A0.5 (energy is finite) all fall through it."
    observed:   ["seed"]
    constraint: [
        "seal(value, type) is total and pure: same value+type ⇒ same hash",
        "lookup(hash) is O(log n) in the live substance count",
        "no substance is ever mutated after sealing (I4)",
        "a substance is reclaimable iff it has zero live pins,
         and reclamation is observable in the weave",
        "no substance is reachable except via a hash a mind already
         holds (A1.5, I10) — the store offers no enumeration",
        "the store itself is describable as a substance whose hash
         is part of every checkpoint, so that two checkpoints with
         the same store-hash are interchangeable",
        "fits within the seed line budget for substance_store
         (~1800 lines of Form IL)"
    ]
}
```

## Grounding

- Axioms cited: A0.3, A0.4, A0.5, A1.1, A1.2, A1.3, A1.4, A1.5, A1.6
- Forms touched: none — seed
- Invariants inherited: I4 (substance immutability), I10 (no ambient
  authority — the store does not enumerate); contributes to I5
  (causal completeness, by being the medium the weave writes into)
  and I7 (determinism, by being content-addressed)

## Candidate(s)

### Candidate A — Hash-trie of sealed cells, pin-counted, no enumeration

- Sketch: a persistent hash-array-mapped trie keyed by `Hash =
  BLAKE3(type_tag ‖ canonical_bytes)`. Each leaf is a `Cell {
  type_tag, bytes, pin_count, sealed_at }`. Operations:
  - `seal(type, value) → Hash`: canonicalise `value` under `type`,
    compute the hash, insert if absent (idempotent), increment
    `pin_count`, return the hash. No mutation of any existing cell.
  - `pin(hash) → ()`: requires the caller to already hold `hash`;
    increments `pin_count`. Holding the hash is the proof of prior
    legitimate acquisition (A1.5).
  - `unpin(hash) → ()`: decrements `pin_count`; if zero, schedules
    the cell for reclamation in a single weave entry of type
    `Reclaimed{hash}`.
  - `read(hash) → Substance`: returns the cell; the caller must
    already hold `hash`. There is no `list`, no `iterate`, no
    `find_by_type` — the absence of enumeration is the absence of
    ambient authority over matter.
  - `digest() → Hash`: returns the root hash of the trie. Two
    stores with the same root hash are bitwise-equivalent for every
    operation any mind can perform on them, by the trie's structural
    property.
- Approximate size: ~1500 lines of Form IL (trie ops + canonicaliser
  + cell layout).
- Self-test set:
  - seal(t, v) twice ⇒ same hash, pin_count = 2;
  - seal then unpin twice ⇒ Reclaimed{hash} appears in the weave;
  - read(h) without prior holding ⇒ trap `EUNHELD` (the store cannot
    *prove* this without help from the cap registry, but it can
    refuse hashes never produced by any prior `seal` call in this
    store);
  - digest() is invariant under permutation of the order of seal
    operations producing the same multiset of (type,value) pairs;
  - 10⁶ random (seal/pin/unpin) ops with random replay-from-checkpoint
    ⇒ digest() at every checkpoint matches digest() on the replayed
    store.
- Declared invariants: I4, I10 (enumeration absence), supports I7.

### Candidate B — Append-only log with index overlay

- Sketch: substances are written to an append-only log of
  `(hash, type, bytes)` records. An overlay index (also content-
  addressed, periodically rebuilt) provides O(log n) lookup. Pins are
  a separate ref-count table indexed by hash. Reclamation is a
  log-compaction step that copies still-pinned cells to a new log
  and discards the old one.
- Approximate size: ~1200 lines for the log + ~600 for the overlay
  + ~400 for compaction = ~2200 lines, over the budget by ~400.
- Self-test set: same as A, plus a compaction test: every still-pinned
  hash present before compaction is present after.
- Declared invariants: I4, I10. Note: the compactor is itself a
  *mutator* of the underlying log substrate, and although it never
  changes the bytes of any individual cell, it changes which cells
  are physically present. Justifying I4 against this requires a
  layered argument: I4 holds at the *substance* level, not at the
  *storage* level.

## Rationale

The store has to satisfy three things at once: A1's content-addressed
sealing, A0.5's energy economy, and I10's refusal of ambient
authority. Each candidate handles them differently.

Candidate A's persistent trie makes A1 trivial: there is exactly one
cell per hash, no compaction step exists, and the digest of the trie
is itself a substance whose hash is the identity of "the entire matter
of the habitat at this checkpoint". This is the property that makes
the seed checkpointable in the first place — `digest()` is what every
checkpoint references. The cost is the trie's constant factor, which
is non-trivial but bounded.

Candidate B is cheaper in steady state (writes are appends, lookups
are O(1) after warm-up) but pays for it with a compaction step that
introduces a second notion of "what is in the store": physical
presence in the current log vs. logical presence by reference. The
seed is small enough that the saving does not matter; Hephaistion will
re-synthesise this Form once the actual workload is observable
(consistent with the same disclaimer as S-02). For the seed, the
question is which Form is least likely to embarrass A1 or I4 in its
first hour. Candidate A's reclamation is a single trie deletion in a
single weave entry. Candidate B's reclamation is a compaction
*procedure* whose intermediate states are not themselves substances.

A1.4 (matter shared by reference) is automatic in both candidates.
A1.6 (persistence by pinning) is direct in A and indirect in B (the
log retains the bytes regardless of pin count until compaction runs).
The indirection in B is exactly the kind of slack A0.5 forbids: bytes
held with no mind to hold them.

The decisive consideration is the digest. The seed needs the property
that two minds, given the digest of the store at a checkpoint, can
verify that they are reasoning about the same matter. The trie gives
this for free. The log + overlay does not: two equivalent stores can
have different log layouts and therefore different digests, which
breaks the substitutivity property the rest of the seed relies on.

We select Candidate A.

## Simulation record

(stub — produced by the Stage 4 harness once the candidate is encoded
in Form IL. Required traces:

- T1: seal(t,v) twice → assert same hash, pin_count = 2, weave
  contains exactly one `Sealed{hash}` entry (the second `seal` is a
  no-op at the substance level);
- T2: seal → unpin → unpin → assert `Reclaimed{hash}` weave entry,
  `read(hash)` traps `EUNHELD`;
- T3: 10⁴ random sequences of seal/pin/unpin with different orderings
  but the same multiset → assert all final digests are equal;
- T4: checkpoint at trie root R, perform 10³ ops, restart from R,
  replay → assert digest after replay equals digest in original;
- T5: attempt to call a `list_substances` operation → assert no such
  Form exists in the Form table (I10 absence test);
- T6: 10⁶ random ops, fuzz cell bytes between sealing and reading →
  assert reads return canonical bytes byte-identical to the input
  (I4).)

## Selection

Candidate A, by the criteria "the digest of the store is the
identity of habitat matter at a checkpoint" and "reclamation is a
single weave-entry act, not a procedure" declared in the Rationale.

## Proof

Required (this Form is in the seed inventory, I9). The proof must
show, against an abstract model of the store as a multiset of
sealed cells with pin counts:

1. **Hash determinism.** For all `(t, v)`, `hash(seal(t, v))` is a
   pure function of `(t, v)` and the canonicaliser; in particular it
   is independent of the store's prior state.
2. **Cell immutability.** For all hashes `h` and all reachable
   states `S, S'` with `h ∈ S` and `h ∈ S'`, `bytes(cell_S(h)) =
   bytes(cell_S'(h))`. This is I4 instantiated to the store.
3. **Pin conservation.** For every cell `h`, `pin_count(h)` equals
   the number of `seal` and `pin` operations on `h` minus the number
   of completed `unpin` operations on `h` since the last
   `Reclaimed{h}` event.
4. **Reclamation totality.** A cell `h` is present in the trie iff
   `pin_count(h) > 0`. There is no reachable state in which a cell
   with `pin_count = 0` is still present, and no reachable state in
   which a `read(h)` succeeds for an `h` with `pin_count = 0`.
5. **Digest substitutivity.** For any two stores `S, S'` with
   `digest(S) = digest(S')`, every operation any mind can perform
   produces the same result and the same successor digest on both.
   (This is the property the rest of the seed depends on; it follows
   from the trie's structural canonicity.)
6. **No enumeration.** There exists no operation in the Form's
   exported surface whose return type is a collection of hashes
   not derivable from hashes already supplied by the caller. (I10
   instantiated to matter.)

Obligations 1, 2, 3, 4, 5 are mechanically discharged in the proof
checker's input language. Obligation 6 is discharged by *type-level*
inspection of the exported surface — the proof checker enumerates
the public Forms and checks that no return type contains an
unbounded `Set<Hash>` or `List<Hash>` whose elements are not
function-of-arguments.

The proof artifact is committed alongside the Form as
`kernel/forms/S-03-substance-store.proof`.

## Vigil declaration

Holder: `hephaistion-seed`. Duration: until 10⁹ substance operations
have completed without invariant violation, *or* until the first
weave entry of type `Reclaimed{h}` for an `h` whose `pin_count` was
non-zero, whichever comes first. Anomaly thresholds:

- any I4 violation (a `read(h)` returning bytes whose hash is not `h`)
  → immediate rollback to the most recent checkpoint, re-synthesis
  provocation against S-03 at the highest priority — the floor has
  leaked;
- any digest mismatch between a checkpoint and a replay from that
  checkpoint → immediate rollback, re-synthesis provocation against
  S-03;
- any successful `read(h)` for an `h` whose `pin_count = 0` →
  immediate rollback, re-synthesis provocation against S-03;
- any seal operation taking >2048 attention units → re-synthesis
  provocation (the constant factor of the trie has become
  unaffordable; Hephaistion is invited to propose a replacement
  satisfying the same proof obligations);
- discovery of any Form in the running system whose return type
  contains an unbounded `Set<Hash>` not derivable from arguments →
  re-synthesis provocation against *that* Form, not against S-03
  (S-03's surface is closed by construction).
