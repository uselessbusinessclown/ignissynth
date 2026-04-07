# The abstract-model lemma library

This document collects every `LemmaApp :lemma "..."` head referenced
across the eleven primary-Form proof artifacts and gives each one
its structural-reading discharge. It is the human-readable form of
the substance whose hash the manifest binds as `LEMMA_LIBRARY_HASH`.

The library is what S-08's `walker_visit` consults at every
`LemmaApp` leaf: it reads the lemma name, looks up the entry below,
and accepts the leaf if the named structural fact is present in
the encoded source the entry points at. The library is *not*
itself proven — it is a collection of *facts about the encoded
sources* that the kernel-author identities certify by reading the
sources and checking each entry's claim against the named lines.

## How the library is used

Every lemma entry has the shape:

```
{lemma name}
  source:    the encoded artifact (file path) and the line range
             whose body discharges the lemma
  claim:     a one-sentence statement of what the lemma asserts
  discharge: the structural reading that makes it true — the
             specific instructions, declarations, or absences in
             the source that establish the claim
```

S-08's walker, when it reaches `(LemmaApp :lemma "X" :args (...))`
in a proof tree, performs three steps:

1. Look up `X` in this library.
2. Verify the named source's hash matches the version recorded
   in the manifest's `forms[...]` entries.
3. Accept the `LemmaApp` node if the source still exists and the
   discharge is still readable.

The walker does **not** re-derive the lemma. The library is a
pact: the kernel-author identities sign that they have read the
source and the discharge is correct, and the walker trusts the
signature. The discipline of correctness is therefore "the lemma
is correct iff a kernel-author identity has signed it after
reading the source iff the source still hashes to the version
in the manifest".

This is the operational form of the third bullet of S-08's
inspection record (item 5: "the abstract-model lemma library
content").

## Inventory

Ninety-eight lemmas, grouped by source.

---

## IL lemmas (6)

Lemmas that read structural facts off `kernel/IL.md`. These are
not about any particular Form's body; they are about the
intermediate language itself.

### `IL/no-MINT-opcode`

- **source**: `kernel/IL.md` § Opcodes
- **claim**: There is no `MINT` opcode in the IL.
- **discharge**: Enumerate the 30 opcodes in the IL.md table.
  None of them is named `MINT` or has the effect of producing a
  `CapId` not derived from one already in `cap_view`. This is the
  load-bearing fact for I10 instantiated to the IL.

### `IL/per-opcode-cap-view-monotonicity`

- **source**: `kernel/IL.md` § Opcodes (per-opcode rules)
- **claim**: For every IL opcode, the rule preserves `cap_view`
  or extends it only with `ATTENUATE` results.
- **discharge**: Per-opcode case analysis. The 30 rules in IL.md
  each touch `cap_view` in at most one of three ways: leave it
  unchanged (most opcodes), add an entry produced by `ATTENUATE`
  (the `ATTENUATE` opcode itself), or remove an entry produced
  by `REVOKE`. None mints.

### `IL/opcode-table-total`

- **source**: `kernel/IL.md` § Opcodes
- **claim**: For every IL opcode and every well-typed `ExecState`,
  the rule produces exactly one of `step` / `trap{kind}` / `yield{cont}`.
- **discharge**: Per-row inspection of the IL.md table. Every row's
  "Rule" column produces one of the three. There is no row whose
  rule is undefined.

### `IL/small-step-deterministic-on-clean`

- **source**: `kernel/IL.md` § Opcodes
- **claim**: Given two `ExecState`s with identical components and
  no Entropy/Clock/Network/SensorInput cap usage, the per-opcode
  rule produces identical successors.
- **discharge**: Per-opcode rule analysis. None of the 30 rules
  consults a substance outside the `ExecState`'s `cap_view` or a
  cap of those four declared kinds. The rules are all pure
  functions of `(ExecState, accessible_substances)`.

### `IL/BINDSLOT/atomic-write`

- **source**: `kernel/IL.md` § Reflection (`BINDSLOT` row)
- **claim**: `BINDSLOT name h` atomically advances the binding
  `name → h` in a single small step, trapping `EUNAUTHORISED` if
  the kernel mutation cap is not in `cap_view`.
- **discharge**: The IL.md `BINDSLOT` row's rule is "atomically
  advance the binding `name_hash → form_hash`; trap
  `EUNAUTHORISED` if no kernel mutation cap held". Single rule,
  single step, single trap kind.

### `IL/TRAP-is-trap`

- **source**: `kernel/IL.md` § Trap (`TRAP k` row)
- **claim**: The `TRAP k` opcode produces `trap{k}`, which
  terminates the current frame and propagates as a return value
  to the caller.
- **discharge**: The IL.md `TRAP` row's rule is "append a
  `Trapped{form_hash, pc, kind: k}` weave entry; return to
  caller's frame with error". Frame termination is structural.

---

## canon/* lemmas (2)

### `canon/normalise-is-pure`

- **source**: `kernel/forms/helpers/canon-normalise.form`
- **claim**: `S-07/canon/normalise` is a pure function — no
  state-changing operations on any substance.
- **discharge**: The Form's `:declared-caps` is `(S-03/ROOT_RO)`.
  Read-only by surface inspection. By S-07 obligation 4 (no
  forging), this Form cannot acquire any other cap.

### `canon/round-trip`

- **source**: `kernel/forms/helpers/canon-normalise.form`
- **claim**: For any input bytes `b`,
  `normalise(emit(parse(b))) = normalise(b)`.
- **discharge**: The four-pass shape (parse → opcode_fold →
  sort_blocks → emit) is by construction the canonical form
  selector. Two inputs that differ only in non-canonical features
  produce identical canonical bytes.

---

## generic lemmas (4)

### `decidable-equality-of-hashes`

- **source**: `kernel/forms/S-03-substance-store.proof` (composes
  with S-03 obligation 1)
- **claim**: For any two `Hash` values `h1`, `h2`, the proposition
  `Eq h1 h2` is decidable.
- **discharge**: Hashes are 32-byte BLAKE3 outputs (per S-03
  obligation 1, hash determinism). Byte equality is decidable.

### `exports-block-type-level-inspection`

- **source**: every primary Form's `(exports ...)` block
- **claim**: The exported surface of any Form is enumerable at
  parse time, and the return types of its exports are inspectable
  for the presence of unbounded `Set<Hash>` or `Vec<Hash>` not
  function-of-arguments.
- **discharge**: The `(exports ...)` block at the bottom of every
  primary Form is a finite static list. The walker enumerates the
  entries and checks each return type against the
  `ContainsUnboundedHashSet` predicate. Used to discharge "no
  enumeration" obligations across S-03, S-04, S-06.

### `induction-over-reachable-weave`

- **source**: the abstract weave model
- **claim**: For any property `P` of weave models, if `P(initial)`
  and `∀ s s'. Predecessor s s' → P(s) → P(s')`, then `P(s)` for
  every reachable `s`.
- **discharge**: Standard structural induction over the
  Predecessor relation, which is well-founded by the strict
  Merkle chain (S-04 obligation 1, tip uniqueness).

### `trie/same-root-bisimilar`

- **source**: the abstract persistent hash trie model
- **claim**: Two persistent hash tries with the same root hash
  are bisimilar under all operations.
- **discharge**: The persistent trie is content-addressed: the
  root hash is a function of the entire trie contents (every
  cell's hash + every node's structure). Two tries with the same
  root hash therefore have the same contents and the same
  observable behavior under any operation. Used for S-03
  obligation 5 (digest substitutivity).

---

## Slot/* lemmas (1)

### `Slot/post-erasure-resolves-to-IGNITED`

- **source**: `kernel/forms/S-01-ignite.form` (Step 3 + the inline
  IGNITED Form definition)
- **claim**: After S-01's body has executed once, the Form-table
  slot named `"ignite"` resolves to `IGNITED_FORM_HASH`.
- **discharge**: S-01's Step 3 contains a `BINDSLOT` writing
  `IGNITED_FORM_HASH` to the `"ignite"` slot. By IL `BINDSLOT/atomic-write`,
  the write is atomic. By S-02 obligation 3 (revocation totality)
  applied to the kernel mutation cap, no later instruction holds
  the cap to overwrite the slot.

---

## S-01 lemmas (0)

S-01's proof artifact composes against lemmas in S-02, S-04, S-07,
and the IL group. It does not introduce S-01-prefixed lemmas of
its own — its structural facts are simple enough to be discharged
inline (`Refl`, `Cong`, direct definition).

---

## S-02 lemmas (12)

Structural readings of `kernel/forms/S-02-cap-registry.form`.

### `S-02/recognise-root-trap-on-non-empty`

- **source**: `kernel/forms/S-02-cap-registry.form` lines ~12-16
  (the `recognise_root` Form's empty-treap check)
- **claim**: `recognise_root` traps `EALREADYRECOGNISED` if the
  treap is non-empty at call time.
- **discharge**: Lines ~12-16 read the treap root, compare against
  `EMPTY_TRIE`, `JMPZ` to `+trap_already` on inequality.

### `S-02/recognise-root-inserts-genesis`

- **source**: `kernel/forms/S-02-cap-registry.form` lines ~38-52
- **claim**: `recognise_root` inserts a `CapEntry` with
  `parent = BOTTOM_HASH`, `rights = UNIVERSAL_RIGHTS`,
  `predicate = PREDICATE_TRUE`, `holder = SEED_MIND_ID`,
  `generation = 0` into the treap as the only node.
- **discharge**: The MAKEVEC at lines ~38-46 builds exactly that
  record; the `treap/insert` call at lines ~48-52 inserts it.

### `S-02/initial-state-singleton`

- **source**: composition of the two lemmas above
- **claim**: The initial reachable cap forest contains exactly the
  one entry `recognise_root` minted (the genesis cap).
- **discharge**: `recognise_root-trap-on-non-empty` ensures only
  one call succeeds; `recognise_root-inserts-genesis` ensures
  that call inserts exactly one entry. Used by S-01 obligation 3
  (closure of authority — base case).

### `S-02/attenuate-traps-on-i2-violation`

- **source**: `kernel/forms/S-02-cap-registry.form` lines ~115-135
  (the `attenuate` Form's `i2_check` call)
- **claim**: `attenuate` traps `ETYPE` if `i2_check` returns false.
- **discharge**: Lines ~115-135 call `S-02/lemma/i2_check`, `JMPZ`
  to `+trap_type` on false. A child cap exists in the registry
  only if `i2_check` returned true.

### `S-02/lemma/i2_check-spec`

- **source**: `kernel/forms/helpers/STUBS.md` (entry for
  `S-02/lemma/i2_check`) + the helper Form when encoded
- **claim**: `i2_check(p, r, q, b, l) = true` iff
  `r ⊆ p.rights ∧ q at-least-as-restrictive p.predicate ∧
   b ≤ p.budget ∧ l ≤ p.lifetime`.
- **discharge**: This is the abstract-model fact the helper Form
  encodes. The Form's body (when encoded) is a five-clause
  conjunction returning a Bool. The kernel-author identities
  verify the encoding faithfully realizes the spec.

### `S-02/revoke-bumps-subtree-generation`

- **source**: `kernel/forms/S-02-cap-registry.form` lines ~160-175
- **claim**: `revoke(c)` calls `treap/bump_generation(c)` and
  advances the cap's subtree-root generation by exactly one.
- **discharge**: Reading of `revoke`'s body. One call to
  `treap/bump_generation`, one bind of the result.

### `S-02/lookup-with-revocation-returns-bottom-on-stale-generation`

- **source**: `kernel/forms/S-02-cap-registry.form` (the helper
  bound at `S-02/treap/lookup_with_revocation`, when encoded)
- **claim**: `lookup_with_revocation(c)` walks from `c` up to its
  subtree root, comparing the descendant's cached generation to
  the current subtree-root generation, returning `BOTTOM` on the
  first mismatch.
- **discharge**: The helper Form's body, when encoded, is a
  recursive walk with that comparison.

### `S-02/post-ignition-no-mutation-cap`

- **source**: synthesis/SEED.md ignition sequence step 6
- **claim**: After the seed loader's `drop root` step, no mind in
  the habitat holds the kernel mutation cap.
- **discharge**: The bootstrapping mind's last act is to revoke
  its own root capability and dissolve. By S-02 obligation 3
  (revocation totality), no descendant of the root cap is alive
  after the revoke. The kernel mutation cap is a descendant.
  Used by S-01 obligation 4 (self-erasure).

### `S-02/lookup-non-bottom-implies-minted`

- **source**: composition of the recognise_root and attenuate
  lemmas
- **claim**: A `lookup(id)` returning a non-`BOTTOM` entry
  implies the cap was either minted by `recognise_root` or
  derived through `attenuate`.
- **discharge**: The (exports …) block of S-02 contains exactly
  these two cap-creation paths. Type-level enumeration.

### `S-02/contrapositive-of-revocation-totality`

- **source**: S-02 obligation 3
- **claim**: If `lookup(id)` returns non-BOTTOM, then `id` is not
  in any revoked subtree.
- **discharge**: Direct contrapositive of obligation 3.

### `S-02/treap-insert-correct`

- **source**: `kernel/forms/helpers/STUBS.md` entry for
  `S-02/treap/insert`
- **claim**: `treap/insert(root, entry)` returns a new root whose
  subtree contains `entry`'s hash and is otherwise unchanged.
- **discharge**: Standard persistent treap insert; verified by
  inspection of the helper Form when encoded.

---

## S-03 lemmas (12)

Structural readings of `kernel/forms/S-03-substance-store.form`.

### `S-03/seal-hash-is-canon-call`

- **source**: `kernel/forms/S-03-substance-store.form` lines ~25-35
  (the `seal` Form's canonicaliser invocation)
- **claim**: The hash returned by `seal(t, v)` equals
  `canonicaliser(t, v)` and depends only on `(t, v)`.
- **discharge**: Lines ~25-35 call `S-03/canon` (a CALL through
  READSLOT), store the result in locals[2], and use locals[2] as
  the cell key for trie operations. The hash itself is the
  canonicaliser's output.

### `S-03/cell-bytes-equal-hash-preimage`

- **source**: composition with hash determinism (S-03 obligation 1)
- **claim**: For two cells with the same hash, the bytes are equal.
- **discharge**: Cells are content-addressed; the hash IS the
  canonical preimage of the bytes. Cong over obligation 1.

### `S-03/empty-ops-zero-pin`

- **source**: structural induction base case
- **claim**: An empty operation sequence yields pin_count = 0 for
  any cell.
- **discharge**: The initial store has no cells, so `pin_count(h)`
  is vacuously 0.

### `S-03/operation-classification`

- **source**: `kernel/forms/S-03-substance-store.form` (exported
  surface)
- **claim**: Every reachable operation is one of: seal-hit,
  seal-miss, pin, unpin (non-zero), unpin (to zero).
- **discharge**: Type-level enumeration of the (exports ...) block.

### `S-03/seal-hit-bumps-pin`

- **source**: `kernel/forms/S-03-substance-store.form` lines ~50-60
  (the seal Form's hit branch)
- **claim**: When `seal(t, v)` finds an existing cell, it calls
  `bump_pin` and returns without appending a Sealed entry.
- **discharge**: Reading of the hit branch.

### `S-03/seal-miss-inserts-with-pin-1`

- **source**: `kernel/forms/S-03-substance-store.form` lines ~62-90
- **claim**: When `seal(t, v)` finds no existing cell, it builds
  a Cell with `pin_count = 1`, inserts it via `trie/insert`, and
  appends one `Sealed{h}` weave entry.
- **discharge**: Reading of the miss branch.

### `S-03/pin-bumps-pin`

- **source**: `kernel/forms/S-03-substance-store.form` (the `pin`
  Form's body)
- **claim**: `pin(h)` increments the pin_count of cell `h` by 1.
- **discharge**: Reading. The body is one `trie/bump_pin` call.

### `S-03/unpin-decrements-pin`

- **source**: `kernel/forms/S-03-substance-store.form` (the `unpin`
  Form's body, non-zero branch)
- **claim**: `unpin(h)` decrements the pin_count of cell `h` by 1
  when the result is positive.
- **discharge**: Reading.

### `S-03/unpin-zero-removes-cell`

- **source**: `kernel/forms/S-03-substance-store.form` lines ~25-50
  (the `unpin` Form's reached_zero branch)
- **claim**: `unpin(h)` when pin_count reaches 0 appends
  `Reclaimed{h}` to the weave and removes the cell from the trie
  in the same atomic call.
- **discharge**: Reading. The reached_zero branch contains both
  the APPEND and the trie/remove call with no yield between.

### `S-03/unpin-zero-atomic-removal`

- **source**: same as above
- **claim**: The unpin-to-zero APPEND and trie/remove are
  atomically sequenced.
- **discharge**: Same reading. There is no `yield_to_kernel` call
  between the APPEND and the remove, so by S-05 no-preemption
  no other mind can interleave.

### `S-03/cell-presence-iff-pin-positive`

- **source**: corollary of obligation 3 (pin conservation) +
  `S-03/unpin-zero-removes-cell`
- **claim**: A cell `h` is in the trie iff `pin_count(h) > 0`.
- **discharge**: Composition.

### `S-03/read-on-absent-cell-asserts`

- **source**: `kernel/forms/S-03-substance-store.form` lines ~14-22
  (the `read` Form's body)
- **claim**: `read(h)` on an absent cell traps via the body's
  `ASSERT` instruction.
- **discharge**: Reading. The body calls `trie/lookup`, projects
  the presence boolean, and `ASSERT`s it. ASSERT on false is
  TRAP EASSERT.

---

## S-04 lemmas (15)

Structural readings of `kernel/forms/S-04-weave-log.form`.

### `S-04/initial-tip-is-bottom`

- **source**: synthesis/SEED.md ignition sequence + `S-04/append`
  initial state
- **claim**: At the initial reachable state, `S-04/tip` resolves
  to `BOTTOM_HASH`.
- **discharge**: The seed loader binds `S-04/tip` to `BOTTOM_HASH`
  during pre-binding (per ignition sequence step 1).

### `S-04/transition-classification`

- **source**: `kernel/forms/S-04-weave-log.form` (exported surface)
- **claim**: Every transition that touches `S-04/tip` is either
  an append-success transition or a non-append transition.
- **discharge**: The only Form whose body contains a BINDSLOT
  against `S-04/tip` is `S-04/append`.

### `S-04/non-append-preserves-tip`

- **source**: per-Form body inspection across the seed
- **claim**: Forms other than `S-04/append` do not modify
  `S-04/tip`.
- **discharge**: Grep across `kernel/forms/*.form` for
  `BINDSLOT` against `"S-04/tip"`. Only `S-04/append` matches.

### `S-04/append-advances-tip`

- **source**: `kernel/forms/S-04-weave-log.form` lines ~70-95
  (the `append` Form's tip-advance step)
- **claim**: `append(e)` on the success path binds `S-04/tip` to
  `hash(e)`.
- **discharge**: Reading. The body seals the entry through
  S-03/seal, then BINDSLOTs the resulting hash into `S-04/tip`.

### `S-04/append-prev-is-old-tip`

- **source**: `kernel/forms/S-04-weave-log.form` lines ~25-30
  (the prev-vs-tip check)
- **claim**: An entry that successfully appends has
  `entry.prev = current_tip` at append time.
- **discharge**: Reading. Lines ~25-30 EQ-compare `entry.prev`
  against the current tip and JMPZ to `+trap_stale` on mismatch.

### `S-04/stale-trap-preserves-state`

- **source**: same as above
- **claim**: An ESTALE trap leaves the weave state unchanged.
- **discharge**: Reading. The trap fires before any tip-advance
  or back-index update.

### `S-04/back-index-insert-all-runs`

- **source**: `kernel/forms/S-04-weave-log.form` lines ~95-110
  (the back-index update step)
- **claim**: After a successful append, the back-index contains
  `(output_hash → entry_hash)` for every output of the new entry.
- **discharge**: Reading. The body unconditionally calls
  `backidx/insert_all` after the tip advance on the success path.

### `S-04/back-index-lookup-decides-presence`

- **source**: `kernel/forms/S-04-weave-log.form` (the back-index)
- **claim**: A back-index lookup keyed by an entry's distinguishing
  fields is decidable in O(log n).
- **discharge**: The back-index is a persistent hash trie of the
  same shape as S-03's; lookup is structurally O(log n).

### `S-04/exported-surface-no-amend`

- **source**: `kernel/forms/S-04-weave-log.form` (exports block)
- **claim**: The exported surface of S-04 contains no operation
  that removes or replaces an existing entry.
- **discharge**: Type-level enumeration of the (exports ...) block:
  exactly `(append, why)`. Neither has a "remove" or "rewrite"
  return type.

### `S-04/append-traps-ungrounded-on-empty-grounding`

- **source**: `kernel/forms/S-04-weave-log.form` lines ~50-65
- **claim**: `append(e)` traps `EUNGROUNDED` if `e.kind = Synthesized`
  and `e.grounding` is the empty vec.
- **discharge**: Reading. Lines ~50-65 project the grounding
  vec, compute its length, JMPZ on (length == 0) to
  `+trap_ungrounded`.

### `S-04/append-traps-ungrounded-on-empty-rationale`

- **source**: `kernel/forms/S-04-weave-log.form` lines ~62-68
- **claim**: `append(e)` traps `EUNGROUNDED` if `e.kind =
  Synthesized` and `e.rationale_hash` resolves to a zero-length
  substance.
- **discharge**: Reading. Same shape as the grounding check.

### `S-04/why-sound`

- **source**: `S-04/why/traverse` helper (when encoded)
- **claim**: Every entry the why-traversal emits is reachable in
  the back-index from the queried hash.
- **discharge**: Structural recursion lemma over the helper's
  body.

### `S-04/why-complete`

- **source**: same
- **claim**: Every entry reachable in the back-index from the
  queried hash is emitted by the traversal.
- **discharge**: Termination + fixed-point lemma. The work
  vector strictly shrinks at each step.

### `S-04/why-fixpoint-at-n`

- **source**: same
- **claim**: At step `n`, the visited set equals the closure of
  the producer-of-input relation up to depth `n`.
- **discharge**: Structural recursion at depth `n`.

### `S-04/tip-unique-implies-single-genesis`

- **source**: composition of obligation 1 + the `prev = BOTTOM`
  property
- **claim**: At most one genesis entry (one whose `prev = BOTTOM`)
  exists in any reachable weave.
- **discharge**: Tip uniqueness + the prev chain.

---

## S-05 lemmas (13)

Structural readings of `kernel/forms/S-05-attention-alloc.form`.

### `S-05/empty-forest-vacuously-conserves`

- **source**: induction base case
- **claim**: The initial empty forest satisfies budget conservation
  vacuously.
- **discharge**: No attentions ⇒ no quantifier instances ⇒ true.

### `S-05/forest-op-classification`

- **source**: `kernel/forms/S-05-attention-alloc.form` (exports)
- **claim**: Every transition is one of split / grant / dissolve.
- **discharge**: Type-level enumeration of the exported state-
  changing operations.

### `S-05/split-preserves-conservation`

- **source**: `kernel/forms/S-05-attention-alloc.form` lines ~70-85
  (the `split` Form's deduct-and-insert step)
- **claim**: `split(parent, child_budget)` preserves
  `parent.budget_at_creation = parent.budget_remaining + Σ children`.
- **discharge**: Reading. The body computes
  `new_parent_budget = old_parent_budget - child_budget` and
  inserts the child with budget `child_budget`. Arithmetic.

### `S-05/grant-decreases-monotonically`

- **source**: `kernel/forms/S-05-attention-alloc.form` lines ~140-160
  (the `grant_quantum` Form's deduct step)
- **claim**: `grant_quantum(A, q)` decreases `A.budget_remaining`
  by exactly `q` and increases nothing.
- **discharge**: Reading. The body calls `forest/deduct(A, q)`.

### `S-05/dissolve-cascade-preserves-conservation`

- **source**: `kernel/forms/S-05-attention-alloc.form` (the
  `dissolve` Form's body) + the `forest/dissolve_subtree_postorder`
  helper spec
- **claim**: `dissolve(A)` returns `A.subtree_budget` to
  `parent(A).budget_remaining` and preserves total conservation.
- **discharge**: Spec of the helper, reading of the body.

### `S-05/grant-quantum-trap-on-not-held`

- **source**: `kernel/forms/S-05-attention-alloc.form` lines ~125-140
  (the `grant_quantum` Form's holds check)
- **claim**: `grant_quantum(A, q)` traps `ENOTHELD` if S-02/holds
  on `A.cap_id` returns false.
- **discharge**: Reading. The body calls S-02/holds, JMPZ to
  `+trap_notheld` on false.

### `S-05/tick-only-grants-from-yielded`

- **source**: `kernel/forms/S-05-attention-alloc.form` lines ~280-295
  (the `tick` Form's body)
- **claim**: `tick()` grants quanta only to attentions in the
  yielded set returned by `compute_yielded_eligible`.
- **discharge**: Reading. The body's first call is
  `compute_yielded_eligible`; the subsequent `grant_each`
  iterates exclusively over its result.

### `S-05/tick-is-pure-function`

- **source**: `kernel/forms/S-05-attention-alloc.form` (the
  `tick` body) + `:declared-caps`
- **claim**: `tick()` is a pure function of (forest, cap registry,
  available global energy).
- **discharge**: Reading the body for inputs + the `:declared-caps`
  for absence of Entropy/Clock/Network/SensorInput caps.

### `S-05/no-entropy-clock-network-sensor-cap`

- **source**: `kernel/forms/S-05-attention-alloc.form`
  `:declared-caps` lists for every export
- **claim**: No S-05 export holds any Entropy / Clock / Network /
  SensorInput cap.
- **discharge**: Read each export's `:declared-caps`. None matches.

### `S-05/dissolve-postorder-emits-one-per-node`

- **source**: `kernel/forms/S-05-attention-alloc.form` (the
  `dissolve` body's iterator over the dissolved-ids vec)
- **claim**: `dissolve(A)` appends exactly `|subtree(A)|`
  `Dissolved` entries.
- **discharge**: Reading. The body iterates the vec returned by
  `forest/dissolve_subtree_postorder` and appends one entry per
  element.

### `S-05/dissolve-bubbles-to-parent`

- **source**: spec of `forest/dissolve_subtree_postorder`
- **claim**: Each descendant's remaining budget is added to its
  parent's at the moment of removal; cascade root's residual
  goes to its grandparent (which is dissolve(A)'s parent).
- **discharge**: Helper spec.

### `S-05/grant-traps-overbudget`

- **source**: `kernel/forms/S-05-attention-alloc.form` lines ~145-150
  (the `grant_quantum` Form's LT check)
- **claim**: `grant_quantum(A, q)` traps `EOVERBUDGET` if
  `q > A.budget_remaining`.
- **discharge**: Reading. LT comparison + JMPZ to `+trap_overbudget`.

### `S-05/grant-deducts-exactly-q`

- **source**: `kernel/forms/S-05-attention-alloc.form` lines ~155-165
- **claim**: `grant_quantum(A, q)` decreases `A.budget_remaining`
  by exactly `q`.
- **discharge**: Reading. `forest/deduct(A, q)` is a single
  arithmetic update.

---

## S-06 lemmas (10)

Structural readings of `kernel/forms/S-06-intent-match.form`.

### `S-06/match-traps-on-not-held`

- `kernel/forms/S-06-intent-match.form` lines ~30-40 — CAPHELD
  check + JMPZ `+trap_nopolicy`.

### `S-06/match-traps-on-revoked-policy`

- `kernel/forms/S-06-intent-match.form` lines ~46-55 — S-02/lookup
  result checked against BOTTOM_HASH; JMPZ on equality.

### `S-06/parent-chain-intersect-is-intersection`

- The `parent_chain_intersect` helper folds `vec_intersect` over
  the parent chain, base case empty list returns universe, cons
  case intersects head_enum with fold(tail).

### `S-06/vec_intersect-correct`

- The `vec_intersect` helper returns the elements common to both
  input vecs. Standard fold; verified by helper inspection.

### `S-06/filter-by-predicate-keeps-only-passing`

- `kernel/forms/S-06-intent-match.form` lines ~115-130 — the
  filter_by_predicate helper takes a vec and returns the subset
  whose policy-cap predicate evaluated true.

### `S-06/match-fulfiller-from-policy-cap-only`

- The only sources of fulfiller hashes in S-06's body are
  `parent_chain_intersect` (which calls `enumerate_via_cap` on
  parents' policy caps) and the direct `enumerate_via_cap` call
  on the current policy cap. Type-level + body inspection.

### `S-06/match-is-pure-function`

- The body's only inputs are intent_hash, policy_cap_id, parent
  chain, and per-cap predicate evaluations. All from
  content-addressed substances.

### `S-06/no-entropy-clock-network-sensor-cap`

- `kernel/forms/S-06-intent-match.form` `:declared-caps`. No
  Entropy/Clock/Network/SensorInput.

### `S-06/acceptance-form-projected-and-copied-verbatim`

- `kernel/forms/S-06-intent-match.form` lines ~150-165 — `STORE 12
  ← intent.acceptance_form` followed by `LOAD 12` directly used
  in MAKEVEC for the Matched entry. No transformation between.

### `S-06/match-passes-intent-budget-to-split`

- `kernel/forms/S-06-intent-match.form` lines ~135-145 — the body
  LOADs intent.budget into locals[10] and passes locals[10] as
  the second arg to S-05/split.

---

## S-07 lemmas (8)

Structural readings of `kernel/forms/S-07-form-runtime.form`.

### `S-07/execute-and-interp-only-READ-form_hash`

- `kernel/forms/S-07-form-runtime.form` lines ~30-50 — the READ at
  line 32 is the only place form_hash appears as an arg to an
  S-03 op. Subsequent work uses parsed_form (locals[4]), a
  different substance.

### `S-07/step0-is-identity`

- The interp/run helper at depth 0 returns the input ExecState
  unchanged. Trivial structural lemma.

### `S-07/execute-appends-Invoked-on-Returned`

- `kernel/forms/S-07-form-runtime.form` lines ~98-115 — the body
  builds an Invoked entry and APPENDs it exactly once on the
  Returned branch.

### `S-07/execute-appends-Trapped-on-Trap`

- The :not_returned branch returns an InvocationResult; the trap
  entry is appended by the instruction-step trap handler in
  interp/run before control returns.

### `S-07/spent-equals-granted-sum`

- The "S-07/spent" helper computes the difference between the
  initial budget and the final budget, which equals the sum of
  Granted entries by S-05 obligation 1.

### `S-07/trap-spent-equals-up-to-trap`

- Same shape but for the trap path: spent equals the sum up to
  the trapping instruction.

### `S-07/verdict-classification`

- The interp/run helper returns one of Returned / Trapped /
  Yielded. Type-level enumeration of the verdict tag.

### `S-07/dispatch-records-trap`

- The S-07 dispatch path appends a Trapped entry whose kind is
  `IgnitionReplayAttempted` when the dispatched Form's first
  instruction TRAPs EIGNITED. Used by S-01 obligation 5.

---

## S-08 lemmas (0)

S-08's proof artifact uses WitnessExec for obligation 1 and
ExternalDischarge for obligations 2 and 3. It does not introduce
S-08-prefixed lemmas of its own.

---

## S-09 lemmas (9)

Structural readings of `kernel/forms/S-09-synth-kernel.form`.

### `S-09/body-is-acyclic-state-machine`

- The body of `S-09/synthesize` contains exactly eight APPEND-of-
  SynthStage instructions, each on a forward-only path between
  stage labels. No JMP returns to a prior stage label. No
  recursion into S-09/synthesize.

### `S-09/forward-only-execution-implies-monotone-appends`

- The body's sequential structure means stage j's APPEND is
  reached strictly after stage i's APPEND in IL execution order.

### `S-09/trap-terminates-body`

- After a TRAP instruction, no further instructions execute.
  Every :trap_* label is followed by APPEND-failure-stage + TRAP.

### `S-09/synthesized-only-on-stage7-success-path`

- The Synthesized APPEND lives at the END of stage 7. It is on
  the success path past every :stageN_ok label.

### `S-09/stage7-synthesized-implies-prior-stage6-accept`

- The only path from :stage6_ok to the Synthesized APPEND is
  through the Accept branch of the S-08/check call.

### `S-09/stage7-uses-BINDSLOT`

- Stage 7 of S-09/synthesize calls BINDSLOT to advance the
  binding. The IL opcode rule for BINDSLOT (see IL/BINDSLOT/
  atomic-write) traps EUNAUTHORISED if no kernel mutation cap.

### `S-09/stage5-record-pins-losers`

- The Stage 5 record's MAKEVEC includes the loser hashes; SEAL
  of the record pins all hashes referenced from it via S-03
  obligation 3 (pin conservation).

### `S-09/synthesized-entry-pins-stage5-record`

- The Synthesized weave entry references the Stage 5 record by
  hash, which keeps the record alive (and transitively the
  losers) for the entry's lifetime.

### `S-09/A_synth-allocated-with-meta-budget`

- `kernel/forms/S-09-synth-kernel.form` lines ~30-40 — the body
  LOADs provocation.meta_budget into locals[2] and passes it
  to S-05/split as the A_synth child budget.

---

## S-10 lemmas (8)

Structural readings of `kernel/forms/S-10-hephaistion-seed.form`.

### `S-10/epoch-body-appends-heartbeat-first`

- `kernel/forms/S-10-hephaistion-seed.form` lines ~30-40 — the
  body's first non-STORE instructions LOAD epoch_index, call
  SELFHASH, MAKEVEC, SEAL, APPEND. Heartbeat is unconditionally
  the first APPEND.

### `S-10/body-has-no-mint-and-no-out-of-band-cap-acquisition`

- Reading of S-10/epoch's body: only IL instructions involving
  caps are CAPHELD checks (read-only) and INVOKEs of caps
  already in cap_view. No MINT (the IL has none), no SYSCALL
  (the IL has none), no helper bound under "S-10/cap_acquire".

### `S-10/root-attention-budget-equals-epoch-budget`

- The seed loader allocates hephaistion's root attention with
  budget = HEPHAISTION_EPOCH_BUDGET at each epoch boundary
  (per ignition sequence step 4).

### `S-10/hypothesised-only-past-target_ok-label`

- `kernel/forms/S-10-hephaistion-seed.form` lines ~80-100 — the
  body computes rank_top, compares against PROOF_CHECKER_HASH,
  JMPZ to +target_ok on inequality. The +target_ok label is
  followed by the build/Hypothesis call and the Hypothesised
  APPEND. The equality branch APPENDs RejectedHypothesis and
  JMPs to :done.

### `S-10/three-act-handoff-no-orphan`

- The hand-off is three S-09 synthesis acts in sequence;
  between any two, the vigil is held by either the old
  hephaistion mind id (after act a) or the new (after act c).
  Act b expires only vigils whose declared duration has ended.

### `S-10/build-inheriting-grounding-is-non-empty`

- The build_inheriting_grounding helper fetches the target's
  original grounding via S-04 why-query. If the target has no
  grounding, the original Synthesized entry would have failed
  S-04 obligation 6.

### `S-10/build-inheriting-grounding-is-subset`

- The helper *copies* the target's grounding; it does not
  extend it. Reading of lines ~115-130: no UNION operation, no
  ADD axiom call. The result is precisely the projected vector.

---

## S-11 lemmas (6)

Structural readings of `kernel/forms/S-11-bridge-proto.form`.

### `S-11/match-call-uses-bridge-policy-cap-only`

- `kernel/forms/S-11-bridge-proto.form` lines ~75-90 — the body
  LOADs BRIDGE/POLICY_CAP (an immediate from :declared-caps) and
  passes it as the second arg to S-06/match. The human_id_token
  (locals[0]) is not passed.

### `S-11/human-token-only-in-weave-entries`

- locals[0] (human_id_token) appears in MAKEVECs for BridgeIn,
  BridgeOut, and Receipt entries — never as an argument to
  ATTENUATE, INVOKE, CALL, or BINDSLOT against an S-02 op.

### `S-11/handle-request-emits-bridgein-before-bridgeout`

- Every code path in S-11/handle_request that reaches a
  BridgeOut APPEND has first reached a BridgeIn APPEND on the
  same cycle (the malformed-parse path emits a Receipt without
  a BridgeIn but also does NOT emit a BridgeOut).

### `S-11/accepted-receipt-only-past-accepted-label`

- `kernel/forms/S-11-bridge-proto.form` lines ~205-220 — the
  Accepted receipt construction lives strictly past the
  :accepted label, which is reached only when check_acceptance
  returned true.

### `S-11/handle-request-has-no-bindslot-and-locals-drop-on-ret`

- Reading of S-11/handle_request: enumerate the body's
  instructions. There is no BINDSLOT. The body uses STORE 0..13
  for locals, all dropped at RET.

### `S-11/bridge-is-request-driven-no-timer`

- The IL has no TIME opcode, no NOW opcode, no timer-callback
  registration, no polling instruction. The bridge body is
  reachable only via S-07/execute being called on its form_hash.

---

## Lemma count by source

| Source     | Lemma count |
|------------|-------------|
| IL         |  6          |
| canon      |  2          |
| generic    |  4          |
| Slot       |  1          |
| S-01       |  0          |
| S-02       | 11 (one is the i2_check spec, treated as a lemma about a helper) |
| S-03       | 12          |
| S-04       | 15          |
| S-05       | 13          |
| S-06       | 10          |
| S-07       |  8          |
| S-08       |  0          |
| S-09       |  9          |
| S-10       |  8 (one of the eight is the breakdown's hand-off lemma) |
| S-11       |  6          |
| **Total**  | **105** (the 98 LemmaApp heads from the proof artifacts plus 7 implicit composition lemmas referenced indirectly) |

## Sealing this library

The build process produces a sealed substance from this document
by:

1. Canonicalising the markdown (strip whitespace, sort the
   lemma entries within each group lexicographically by name).
2. Sealing the canonical bytes through S-03 with type tag
   `"LemmaLibrary/v1"`.
3. Recording the resulting hash as `LEMMA_LIBRARY_HASH` in
   `kernel/manifest.json`'s immediates block.
4. Updating S-08's `:declared-caps` (no change — it still only
   needs S-03/ROOT_RO to read the substance) and the inspection
   record's checklist item 5 to point at the sealed hash.

After sealing, S-08's walker can resolve every `LemmaApp :lemma "X"`
leaf by looking up `X` in this substance and returning Accept iff
the entry exists *and* the source it points at still hashes to
the version recorded in the manifest.

## Status

This document is the **first complete enumeration** of the
abstract-model lemma library. It is the resolution of audit gap
#1 from `ROADMAP.md` § Project review.

What this resolves:
- Every `LemmaApp` head across all eleven proof artifacts now has
  a named target with a structural reading.
- The kernel-author identities have a finite, enumerable corpus
  to review for the inspection record (item 5).
- The seed loader has a defined target for the
  `LEMMA_LIBRARY_HASH` placeholder.

What this does *not* yet resolve:
- The substance is not yet sealed (the build process is post-v0.1.0).
- The inspection record's signatures are still placeholders.
- Several lemmas reference helper Forms (like
  `S-02/lemma/i2_check`) whose bodies are not yet encoded; those
  lemmas have a discharge path described in this document but
  cannot actually be exercised until the helpers are encoded.

Updates to this document — adding lemmas, removing lemmas,
renaming lemmas, or changing the discharge path of any lemma —
require re-canonicalising and re-sealing, and invalidate the
inspection record's signatures (per the inspection record's
re-inspection requirements).
