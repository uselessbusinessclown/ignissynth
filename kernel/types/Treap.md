# `Treap` — persistent treap for `S-02/cap-registry`

> Sister document to `kernel/types/SCHEMA.md`. Specifies the node
> layout of the persistent treap chosen as **Candidate A** in
> `breakdown/S-02-cap-registry.md`. The helpers under
> `S-02/treap/*` in `kernel/forms/helpers/STUBS.md` walk these
> node layouts.
>
> The treap stores `CapEntry/v1` substances keyed by `CapId` (a
> `Hash`). It is the registry the cap predicate Form looks
> entries up in, and the structure revocation operates on. A
> change to any layout below is a synthesis act subject to I9.

## Design principles

Inherited from `SCHEMA.md`. Treap-specific:

1. **BST key + max-heap priority.** Nodes are ordered as a BST
   on `CapId` (lexicographic on the 32 hash bytes) and as a
   max-heap on `priority`. The priority is deterministic from
   the key (see below), so two treaps holding the same
   `(CapId → CapEntry)` multiset have the same shape and the
   same root hash. This is the canonicity property
   `breakdown/S-02` § Proof obligation 4 ("lookup totality")
   composes against.
2. **Persistent.** Every operation returns a new root; no
   existing node is ever mutated.
3. **Bounded depth.** Expected depth O(log n); worst-case
   bounded by the keys, not by descendant count
   (`breakdown/S-02` § Rationale).
4. **Revocation is generation, not deletion.** A revoked
   `CapEntry` is *still in the treap*. Its `generation` field
   has been bumped, and `lookup_with_revocation` compares the
   stored generation against the parent's generation walking
   up the derivation chain. Revocation is therefore O(log n),
   not O(descendants). The cost is paid at lookup time.

### Priority derivation

`priority` is the first 8 bytes of `BLAKE3(CapId ‖ "treap-priority")`
interpreted as `Nat(8)`. This:

- gives a uniform distribution over `Nat(8)` regardless of how
  `CapId` was constructed,
- is deterministic — two implementations agree on the priority
  of every key without coordination,
- is independent of the cap-derivation chain (so derivation
  patterns can't tilt the tree).

The priority is **not** stored in the `CapEntry/v1` substance.
It is stored in the treap node so the structure is
self-describing under inspection.

## Node kinds

Two node shapes, both sealed substances:

| Kind   | Type tag           | Purpose                              |
|--------|--------------------|--------------------------------------|
| Empty  | `"TreapEmpty/v1"`  | The empty treap. Singleton.          |
| Branch | `"TreapBranch/v1"` | One CapEntry + left/right subtrees.  |

A `TreapRoot` is the `Hash` of either of the above. The
registry's logical state is named by the root hash.

### `TreapEmpty/v1`

| Field      | Offset | Type        | Meaning                          |
|------------|--------|-------------|----------------------------------|
| `type_tag` | 0      | `Bytes(13)` | literal `"TreapEmpty/v1"`        |

Total size: 13 bytes. The hash is constant across the seed —
pinned in the manifest as `EMPTY_TREAP` (placeholder
`$$BLAKE3$$/empty-treap` until the cold-weave seal).

### `TreapBranch/v1`

| Field         | Offset | Type        | Meaning                                                          |
|---------------|--------|-------------|------------------------------------------------------------------|
| `type_tag`    | 0      | `Bytes(14)` | literal `"TreapBranch/v1"`                                       |
| `cap_id`      | 14     | `Hash(32)`  | the BST key                                                      |
| `priority`    | 46     | `Nat(8)`    | derived per `priority = first 8 bytes of BLAKE3(CapId ‖ "treap-priority")` |
| `cap_entry`   | 54     | `Hash(32)`  | hash of the `CapEntry/v1` substance at this node                 |
| `left`        | 86     | `Hash(32)`  | hash of the left subtree (`TreapEmpty/v1` or `TreapBranch/v1`)   |
| `right`       | 118    | `Hash(32)`  | hash of the right subtree (`TreapEmpty/v1` or `TreapBranch/v1`)  |

Total size: 150 bytes.

Invariants enforced by helpers (trap `EBADLOCAL` on violation):

- BST: every key in `left` subtree < `cap_id` < every key in
  `right` subtree (lexicographic on hash bytes).
- Heap: this node's `priority` ≥ the `priority` of `left` and
  `right` Branch nodes (Empty nodes are below every priority).
- `priority == first 8 bytes of BLAKE3(cap_id ‖ "treap-priority")`.
  The helper recomputes this on read; mismatch is `EBADLOCAL`.
  This is what makes the priority untrusted-data-safe.
- `cap_entry` points at a `CapEntry/v1` substance whose
  `holder` and `generation` fields are read by the projection
  helpers in `S-02`.

## Operations

All operations are persistent. Implementations live (eventually)
in `kernel/forms/helpers/treap.form`.

### `S-02/treap/insert`

```text
(TreapRoot, CapEntry) → TreapRoot
```

- The CapEntry's `cap_id` is computed by the caller from the
  cap preimage; `insert` does not derive it.
- BST-inserts `(cap_id, cap_entry)` by key, then rotates up the
  tree until the heap property is restored.
- If `cap_id` is already present, the new `cap_entry` *replaces*
  the old one. This is how `attenuate` adds children: a child
  CapEntry's `cap_id` is `BLAKE3(parent_cap_id ‖ child_rights ‖
  child_predicate)`, so the parent and child have different
  `cap_id`s by construction; replacement at insert-time only
  happens for genuinely-same-id re-mints, which the proof of
  obligation 1 ("mint uniqueness") rules out.
- Traps: `ETYPE` on any malformed node; never `EOVERBUDGET`.

### `S-02/treap/lookup`

```text
(TreapRoot, CapId) → CapEntry
```

- BST walk by `cap_id`; returns the stored `cap_entry` hash on hit.
- On miss, returns `BOTTOM_HASH`. This is a total function: it does
  **not** trap. The frozen `S-02/attenuate` Form calls it and
  immediately compares the result against `BOTTOM_HASH`
  (`→ CapEntry or BOTTOM`), and its `declared-traps` is
  `(ENOTHELD ETYPE)` — there is no `EUNHELD` to propagate.
- Step cost: O(log n) expected, O(n) worst-case bounded by
  treap height.

### `S-02/treap/lookup_with_revocation`

```text
(TreapRoot, CapId) → CapEntry
```

The contract `S-02/lookup` (and through it `S-02/holds`) calls
into. It is a **total function** — it returns `BOTTOM_HASH`, never
traps, because `S-02/lookup` wraps it with `declared-traps ()` and
returns the result directly, and `S-02/holds` then compares against
`BOTTOM_HASH`:

1. BST-walk to find the node with `cap_id`; if none, return
   `BOTTOM_HASH`.
2. Walk up the cap's derivation chain (via `CapEntry.parent`)
   re-doing a `lookup` on each parent, comparing the stored
   `generation` against the value the child recorded at the
   time of attenuation.
3. If any ancestor's stored generation is higher than the
   recorded one, the cap is revoked ⇒ return `BOTTOM_HASH`.
4. Otherwise return the matched `cap_entry`.

The walk is bounded by the cap's derivation depth, which `S-02`
obligation 2 ("attenuation monotonicity") bounds.

> **Note.** There is no `EREVOKED` trap kind. The IL trap
> enumeration (`kernel/IL.md` § Trap kinds) is closed at eleven
> kinds; revocation is signalled by the `BOTTOM_HASH` return value,
> not by a trap. An earlier draft of this document named a
> nonexistent `EREVOKED` trap — corrected when `treap.form` was
> encoded against this spec.

### `S-02/treap/bump_generation`

```text
(TreapRoot, CapId) → TreapRoot
```

- Locates the entry for `cap_id` and produces a new
  `CapEntry/v1` substance with `generation += 1`. Inserts the
  new entry over the old one (same `cap_id`, replacement).
- Returns the new root hash.
- This is how `S-02/revoke` propagates revocation in O(log n):
  every descendant's next `lookup_with_revocation` will compare
  its recorded generation against this bumped one and return
  `BOTTOM_HASH`.
- Absent key: returns the root unchanged (the identity). The sole
  caller `S-02/revoke` guards with `CAPHELD` and declares only
  `(ENOTHELD)`, so a held cap is always present and the absent
  branch is unreachable in practice; it is the identity rather than
  a trap so no trap kind outside `revoke`'s declaration can escape.

## Canonicity proof sketch

Two `TreapRoot`s have the same hash iff they encode the same
`(CapId → CapEntry)` multiset. Sketch:

- The BST shape is fully determined by the set of keys and the
  priorities (heap property + BST property uniquely determine
  the tree).
- Priorities are deterministic from keys (principle 1).
- Therefore the same multiset of `(key, value)` pairs always
  produces the same tree shape, and hashing a tree is
  bottom-up, so the root hash is determined.

`breakdown/S-02` § Proof obligation 4 cites this directly.

## Status

This document is the v0.2.0-helpers layout reference for the
treap operations. It is sufficient for `S-02/treap/*` helpers
to be encoded against. The Rust implementation lives only in
`ignis0` (the stage-0 substrate); the seed itself walks the
treap by helper Form calls, not by host-language primitives.

Open issues this spec deliberately does *not* settle:

- **Cap-revocation breadth.** The current design pays for
  revocation at lookup time, which keeps `revoke` O(log n) at
  the cost of O(depth) walks on every authority check. A
  future re-synthesis of S-02 (Hephaistion's job per `S-10`)
  may choose to amortize differently.
- **In-place priority salt rotation.** The `"treap-priority"`
  salt string is hard-coded. If an adversary ever discovers a
  way to manipulate `CapId`s into a degenerate tree, the salt
  can be rotated, but that's a re-synthesis act (every existing
  CapEntry has to be re-inserted under the new salt). The
  bootstrap-lock on S-02 means this can't happen accidentally.
