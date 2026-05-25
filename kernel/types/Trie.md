# `Trie` — persistent hash-array-mapped trie for `S-03/substance-store`

> Sister document to `kernel/types/SCHEMA.md`. Specifies the node
> layout of the persistent hash-array-mapped trie (HAMT) chosen
> as **Candidate A** in `breakdown/S-03-substance-store.md`. The
> helpers under `S-03/trie/*` in `kernel/forms/helpers/STUBS.md`
> walk these node layouts; `ignis0`'s `v0.2.5-ignis0-store`
> milestone replaces the HashMap-backed `SubstanceStore` with an
> implementation of this spec.
>
> A change to any layout below is a synthesis act subject to I9
> (every change to type bytes requires a re-seal). Version tag
> bumps must be paired with a re-seal of every substance whose
> tag changes.

## Design principles

Inherited from `SCHEMA.md`:

1. **Self-describing.** Every node carries its type tag as the
   first field.
2. **Fixed offsets.** Within a versioned tag, every field has a
   fixed byte offset.
3. **Variable-length tail.** Only the last field is variable-length.
4. **Hash references, not values.** Sub-nodes are referenced by
   `Hash`; the helper never embeds a sub-node inline.

HAMT-specific:

5. **Branching factor 32 (5-bit chunks of the key hash).** The key
   space is `Hash` (32 bytes = 256 bits = 51 full chunks of 5 bits
   with one 1-bit remainder). Trees are at most 52 levels deep in
   the worst case; in practice O(log₃₂ n) levels for n cells.
6. **Bitmap-compressed branch nodes.** A branch node stores only
   the children that exist, indexed by a 32-bit `bitmap`. Slot
   index in the `children` array for set bit `b` is
   `popcount(bitmap & ((1 << b) - 1))`. This is the standard
   Bagwell HAMT layout — well understood, audit-friendly, and
   gives the structural canonicity property the digest depends on.
7. **Total functions over the layout.** Every helper that reads a
   trie node verifies the type tag first and traps `ETYPE` on
   mismatch. `EBADLOCAL` is the trap for malformed branch bitmaps
   (e.g. `popcount(bitmap) != len(children)`).
8. **Structural canonicity.** Two tries holding the same
   `(hash → cell)` multiset have the same root hash, regardless of
   insertion order. This is the property that makes
   `S-03/digest()` substitutive (per `breakdown/S-03` § Rationale).

## Node kinds

A trie node is one of three shapes, distinguished by its
`type_tag`. All three are sealed substances (referenced by `Hash`).

| Kind   | Type tag         | Purpose                                    |
|--------|------------------|--------------------------------------------|
| Empty  | `"TrieEmpty/v1"` | The empty trie. Singleton substance.       |
| Branch | `"TrieBranch/v1"`| Interior node with up to 32 children.      |
| Leaf   | `"TrieLeaf/v1"`  | A `(key, cell)` pair at the bottom.        |

A `TrieRoot` is just the `Hash` of any of the above. The store's
`digest()` is the hash of the current root substance.

### `TrieEmpty/v1`

The singleton empty trie. The store starts here. Every fresh
sub-tree that collapses to nothing reverts to this substance.

| Field      | Offset | Type        | Meaning                          |
|------------|--------|-------------|----------------------------------|
| `type_tag` | 0      | `Bytes(12)` | literal `"TrieEmpty/v1"`         |

Total size: 12 bytes. Hash is constant across the seed — pinned
in the manifest as `EMPTY_TRIE_HASH` (placeholder
`$$BLAKE3$$/empty-trie` until the cold-weave seal).

### `TrieBranch/v1`

An interior node with up to 32 children, one per 5-bit slot of
the key hash at the current depth.

| Field        | Offset | Type        | Meaning                                                    |
|--------------|--------|-------------|------------------------------------------------------------|
| `type_tag`   | 0      | `Bytes(13)` | literal `"TrieBranch/v1"`                                  |
| `depth`      | 13     | `Nat(1)`    | which 5-bit chunk of the key this node tests (0..=51)      |
| `bitmap`     | 14     | `Nat(4)`    | bit `b` set ⇒ child for slot `b` is present                |
| `pin_sum`    | 18     | `Nat(8)`    | sum of `pin_count` across every leaf in this subtree       |
| `children_n` | 26     | `Nat(1)`    | `popcount(bitmap)`; redundant but pinned for cheap verify  |
| `children`   | 27     | `Hash[]`    | child node hashes in slot order (length = `children_n`)    |

Total size: `27 + 32 * popcount(bitmap)`. Range: `27 + 32 = 59`
(one child) to `27 + 32*32 = 1051` (full node).

Invariants enforced by helpers (trap `EBADLOCAL` on violation):

- `children_n == popcount(bitmap)`.
- `children_n >= 1`. A branch with zero children must be
  rewritten as `TrieEmpty/v1`.
- `children_n != 1` *unless* the lone child is itself a Branch —
  a Branch with a single Leaf child collapses to that Leaf.
  (This is the canonicity-preserving collapse rule; without it,
  insertion order would leak into the structure.)

`pin_sum` exists so `decr_pin` can short-circuit subtrees with
zero live cells without descending into them, and so the
free-bytes accounting (A0.5) can be computed in O(1) at the root.

### `TrieLeaf/v1`

A leaf substance holding one `(key, cell)` pair.

| Field        | Offset | Type        | Meaning                              |
|--------------|--------|-------------|--------------------------------------|
| `type_tag`   | 0      | `Bytes(11)` | literal `"TrieLeaf/v1"`              |
| `key`        | 11     | `Hash(32)`  | the cell's content hash              |
| `cell`       | 43     | `Hash(32)`  | hash of the `Cell/v1` substance      |
| `pin_count`  | 75     | `Nat(8)`    | number of live pins on this cell     |

Total size: 83 bytes.

The `cell` field references a `Cell/v1` substance (the actual
sealed bytes). The trie does not embed the cell; that would
violate principle 4 above and inflate the trie's node bytes.

### `Cell/v1`

Not a trie node, but the substance every `TrieLeaf/v1.cell`
points at. Documented here because the leaf's contract refers
to it.

| Field         | Offset | Type        | Meaning                              |
|---------------|--------|-------------|--------------------------------------|
| `type_tag`    | 0      | `Bytes(7)`  | literal `"Cell/v1"`                  |
| `inner_tag_n` | 7      | `Nat(4)`    | length of the wrapped type tag       |
| `inner_tag`   | 11     | `Bytes[]`   | the wrapped substance's type tag     |
| `sealed_at`   | tail-8 | `Nat(8)`    | weave-entry index at first seal      |
| `bytes_n`     | tail-4 | `Nat(4)`    | length of the wrapped bytes          |
| `bytes`       | tail   | `Bytes[]`   | the wrapped substance bytes          |

`key` in the leaf is `BLAKE3(inner_tag ‖ bytes)` per A1.1, so the
leaf's `key` is determined by `cell` — the helper trusts this
identity and does not recompute on every read.

## Operations

Each helper below has an entry in `kernel/forms/helpers/STUBS.md`.
This section pins the contract — input/output shape, the trap
kinds the helper may produce, and the canonical structural
behavior. The implementations live (eventually) in
`kernel/forms/helpers/trie.form`.

All operations are **persistent**: they return a new root hash;
no existing node is ever mutated. Sharing is automatic via
content-addressing — unchanged subtrees keep their hash.

### `S-03/trie/lookup`

```text
(TrieRoot, Hash) → Pair{Bool, Cell}
```

- Walks from `TrieRoot` down by 5-bit chunks of the key.
- If a `TrieEmpty/v1` is reached, returns `Pair{false, BOTTOM_HASH}`.
- If a `TrieLeaf/v1` with matching `key` is reached, returns
  `Pair{true, leaf.cell}`.
- If a `TrieLeaf/v1` with non-matching `key` is reached, returns
  `Pair{false, BOTTOM_HASH}`.
- Traps: `ETYPE` if any node visited fails its type-tag check.
- Step cost: O(depth) ≤ 52 substance reads.

### `S-03/trie/insert`

```text
(TrieRoot, Hash, Cell) → TrieRoot
```

- If the key is already present, returns `TrieRoot` unchanged
  and the leaf's `pin_count` is *not* bumped here — pinning is
  `bump_pin`'s job, and `S-03/seal` composes the two.
- If the key is absent, allocates a new `TrieLeaf/v1` with
  `pin_count = 0` (composed `seal` will bump it to 1).
  When a Branch's child needs to split (existing leaf + new leaf
  share a prefix), descends to the first differing 5-bit chunk,
  creating intermediate Branches as needed.
- Traps: `ETYPE` on any malformed node; never `EOVERBUDGET` (the
  budget check is the caller's responsibility).

### `S-03/trie/bump_pin`

```text
(TrieRoot, Hash) → TrieRoot
```

- Increments `pin_count` on the matching leaf and updates
  `pin_sum` on every Branch on the path.
- If the key is absent, traps `EBADLOCAL`. The caller is
  required to have just inserted the leaf or to know it exists.
- Persistent: the new root shares every untouched subtree with
  the old root.

### `S-03/trie/decr_pin`

```text
(TrieRoot, Hash) → Pair{TrieRoot, Bool}
```

- Decrements `pin_count` on the matching leaf.
- Returns `Pair{new_root, true}` if `pin_count` reached zero
  (caller emits the `Reclaimed{hash}` weave entry).
- Returns `Pair{new_root, false}` otherwise.
- If `pin_count` was already zero, traps `EUNDERFLOW`.

### `S-03/trie/remove`

```text
(TrieRoot, Hash) → TrieRoot
```

- Removes the leaf for `key` and collapses single-child
  Branches per the canonicity rule.
- If the key is absent, returns `TrieRoot` unchanged (idempotent
  per A1.4: removing what isn't there is a no-op).
- Composed by `S-03/seal` after `decr_pin` returns `Bool=true`.

## Digest

`S-03/digest()` is exactly the root substance's hash. Two stores
that contain the same `(hash → Cell)` multiset have identical
root hashes by the canonicity rule (principle 8). This is the
property the checkpointing logic depends on; `breakdown/S-03`
§ Proof obligation 1 ("hash determinism") composes against it
directly.

## Self-test vector (for `ignis0` v0.2.5-store)

When the host-language implementation of this spec lands in
`ignis0/src/store.rs`, the following test must pass:

1. `seal(t, v)` twice ⇒ same hash, leaf `pin_count == 2`, root
   hash stable across the two operations after the second.
2. `seal then unpin twice` ⇒ `Reclaimed{hash}` appears in the
   weave log; root hash returns to a state where `lookup(h)`
   returns `Pair{false, BOTTOM_HASH}`.
3. `digest()` invariant under permutation: `seal(a); seal(b);
   seal(c)` and `seal(c); seal(b); seal(a)` produce identical
   root hashes.
4. 10⁶ random `(seal/pin/unpin)` ops with random
   replay-from-checkpoint: root hash at every checkpoint matches
   root hash on the replayed store.

The fixed-point check (A9.3) does *not* exercise this — it tests
the interpreter, not the store — but the simulation harness
(Stage 4) will.

## Status

This document is the v0.2.0-helpers layout reference for the
trie operations. It is sufficient for `S-03/trie/*` helpers to be
encoded against, and for `ignis0/src/store.rs` to be rewritten
against in milestone `v0.2.5-ignis0-store`.

Open issues this spec deliberately does *not* settle:

- **Collision handling.** With BLAKE3 keys and 256-bit hash space,
  full-collision Leaf nodes (separate `(key, cell)` pairs that
  share all 51 5-bit chunks) are cryptographically unreachable —
  one would predate the heat death. The spec therefore does not
  define a `TrieCollision/v1` node. If a future audit demands
  one, the type tag is reserved.
- **Concurrent writers.** The trie is persistent; concurrent
  readers and writers see consistent snapshots by construction.
  Coordination between writers (which root wins on conflict) is
  `S-09/synth_kernel`'s problem, not this spec's.
- **Compaction / garbage collection.** None. A reclaimed cell is
  literally removed from the trie; the substance store does not
  hold bytes whose pin count is zero. This is the design choice
  that distinguishes Candidate A from Candidate B in `breakdown/S-03`.
