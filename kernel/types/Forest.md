# `Forest` — persistent attention forest for `S-05/attention-alloc`

> Sister document to `kernel/types/SCHEMA.md`. Specifies the
> persistent attention forest chosen as **Candidate A** in
> `breakdown/S-05-attention-alloc.md`. The helpers under
> `S-05/forest/*` in `kernel/forms/helpers/STUBS.md` walk this
> structure.
>
> The forest is the global state of attention: every live
> `Attention` substance and its parent-child relationships.
> `AttentionRecord/v1` (defined in `SCHEMA.md`) is the per-node
> payload; this document specifies how the records are organised
> into a navigable persistent forest. A change to any layout
> below is a synthesis act subject to I9.

## Why a forest, not a tree

A3.2 says every mind owns a *tree* of attentions; A3.5 says
multiple minds coexist. The seed therefore has many trees, one
per mind, with no inter-mind structural sharing — a forest.

Concretely the forest is a map `AttId → AttentionRecord` plus an
auxiliary index for parent → children navigation. The map is
the persistent structure that gets new roots after each
operation; the per-`AttId` reverse-link is reconstructed
on-demand from a node's children list (stored in the record).

## Design principles

Inherited from `SCHEMA.md`. Forest-specific:

1. **Persistent map keyed by `AttId`.** `AttId` is a `Hash` —
   the same key space as the substance store. The forest is a
   HAMT instance (see `Trie.md`) parameterised on a different
   value type: leaves carry `AttentionRecord/v1` hashes instead
   of `Cell/v1` hashes.
2. **Children stored on the parent.** Each
   `AttentionRecord/v1` carries a `Vec{AttId}` of its direct
   children, ordered by creation. Postorder dissolution walks
   this list bottom-up.
3. **Atomicity at the API surface.** `atomic_split` and
   `dissolve_subtree_postorder` are single Form calls that
   produce a single new `ForestRoot`. Half-applied splits are
   structurally impossible — there is no public helper that
   produces an intermediate forest state.
4. **Determinism by node id.** The operations are total and
   deterministic functions of `(ForestRoot, AttId, …)`. Two
   minds applying the same operation sequence to the same
   starting forest produce the same final root hash. This is
   what `breakdown/S-05` § Proof obligation "determinism"
   composes against.

## Node kinds

The forest reuses the HAMT structure from `Trie.md` with a
parallel set of type tags so the same helper code can
parameterise:

| Kind   | Type tag            | Purpose                                    |
|--------|---------------------|--------------------------------------------|
| Empty  | `"ForestEmpty/v1"`  | The empty forest. Singleton substance.     |
| Branch | `"ForestBranch/v1"` | Interior node, up to 32 children.          |
| Leaf   | `"ForestLeaf/v1"`   | An `(att_id, record)` pair at the bottom.  |

A `ForestRoot` is a `Hash` of any of the above. There is no
distinguished "global root" node — the forest is a single map;
which `AttId`s are roots of *attention trees* is encoded in
the records (an `AttentionRecord` whose `parent ==
BOTTOM_HASH` is a tree root).

### `ForestEmpty/v1`

| Field      | Offset | Type        | Meaning                          |
|------------|--------|-------------|----------------------------------|
| `type_tag` | 0      | `Bytes(14)` | literal `"ForestEmpty/v1"`       |

Pinned in the manifest as `EMPTY_FOREST` (placeholder
`$$BLAKE3$$/empty-forest` until the cold-weave seal).

### `ForestBranch/v1`

| Field         | Offset | Type        | Meaning                                                    |
|---------------|--------|-------------|------------------------------------------------------------|
| `type_tag`    | 0      | `Bytes(15)` | literal `"ForestBranch/v1"`                                |
| `depth`       | 15     | `Nat(1)`    | which 5-bit chunk of the AttId this node tests (0..=51)    |
| `bitmap`      | 16     | `Nat(4)`    | bit `b` set ⇒ child for slot `b` is present                |
| `budget_sum`  | 20     | `Nat(8)`    | sum of `budget_remaining` across every leaf in this subtree |
| `children_n`  | 28     | `Nat(1)`    | `popcount(bitmap)`                                         |
| `children`    | 29     | `Hash[]`    | child node hashes in slot order                            |

Total size: `29 + 32 * popcount(bitmap)`.

`budget_sum` enables the I6 ("bounded epoch budget") check at the
root in O(1): the forest's total live budget never exceeds the
mint event's budget allocation.

Same canonicity-collapse rule as `TrieBranch/v1` (single-leaf
Branches collapse to the Leaf).

### `ForestLeaf/v1`

| Field      | Offset | Type        | Meaning                                                  |
|------------|--------|-------------|----------------------------------------------------------|
| `type_tag` | 0      | `Bytes(13)` | literal `"ForestLeaf/v1"`                                |
| `att_id`   | 13     | `Hash(32)`  | this attention's id                                      |
| `record`   | 45     | `Hash(32)`  | hash of the `AttentionRecord/v1` substance               |
| `yielded`  | 77     | `Hash(32)`  | continuation hash if yielded, else `BOTTOM_HASH`         |

Total size: 109 bytes.

`yielded` is duplicated from the inner record so YIELD/resume
can be detected without dereferencing `record`. The helpers
keep the two in sync; a divergence is `EBADLOCAL`.

## Operations

All operations are persistent. Implementations live (eventually)
in `kernel/forms/helpers/forest.form`.

### `S-05/forest/get`

```text
(ForestRoot, AttId) → AttentionRecord
```

- HAMT lookup by `att_id`.
- On hit, returns the `record` hash from the matching leaf.
- On miss, traps `EUNHELD`.

### `S-05/forest/new_child`

```text
(AttId, Nat, CapId) → AttId
```

A pure derivation function — no `ForestRoot` argument. Returns
the deterministic `AttId` a child would have under
`(parent_att_id, child_index, focus_cap_id)`:

`child_att_id = BLAKE3("AttId" ‖ parent_att_id ‖ uleb128(child_index) ‖ focus_cap_id)`

The seed never mints `AttId`s by RAND; every `AttId` is a
function of its parent's `AttId` plus its index plus its focus.
This makes the entire attention forest replayable from the
sequence of `Split` weave entries.

### `S-05/forest/atomic_split`

```text
(ForestRoot, AttId, Nat, AttId, Nat) → ForestRoot
```

Atomic version of "remove this much budget from parent + create
this child with that much budget":

- Args: `(ForestRoot, parent_id, child_id, child_budget, child_deadline)`.
- Looks up parent. If not present: trap `EUNHELD`.
- If `parent.budget_remaining < child_budget`: trap `EOVERBUDGET`.
- Allocates a new `AttentionRecord/v1` for the child with
  `parent = parent_id`, `mind_id = parent.mind_id`,
  `cap_id = parent.cap_id` (children inherit the parent's
  focus by default — re-focusing is a separate operation),
  `cap_view = parent.cap_view`, `budget_remaining = child_budget`,
  `deadline = child_deadline`.
- Allocates a new `AttentionRecord/v1` for the parent with
  `budget_remaining -= child_budget` and `children` appended
  with `child_id`.
- Inserts both new leaves into the HAMT and returns the new
  `ForestRoot`.
- Atomicity: there is no public helper that produces a
  half-applied state. Either both updates land or none do.

### `S-05/forest/dissolve_subtree_postorder`

```text
(ForestRoot, AttId) → Pair{ForestRoot, Vec{AttId}}
```

- Walks the subtree rooted at `att_id` in postorder (children
  before parent).
- Removes each visited record from the forest map.
- Returns the new root and the `Vec{AttId}` of removed ids in
  the order they were removed.
- The caller emits one `Dissolved{att_id}` weave entry per id
  returned (proof obligation S-05 #4 "dissolution accounting").

### `S-05/forest/deduct`

```text
(ForestRoot, AttId, Nat) → ForestRoot
```

- Reads the record at `att_id`.
- If `budget_remaining < amount`: trap `EOVERBUDGET`.
- Produces a new record with `budget_remaining -= amount`,
  re-inserts, returns the new root.

### `S-05/forest/mark_yielded`

```text
(ForestRoot, AttId, Hash) → ForestRoot
```

- Sets the leaf's `yielded` field to the continuation hash.
- Also updates the inner `AttentionRecord/v1` so the two stay
  in sync (`yielded` is duplicated on the leaf for cheap lookup,
  see `ForestLeaf/v1` above).
- Traps: `EUNHELD` if `att_id` absent.

### `S-05/forest/set_deadline`

```text
(ForestRoot, AttId, Nat) → ForestRoot
```

- Reads the record, produces a new one with the new `deadline`,
  re-inserts.
- Deadlines are weave-entry-count offsets; the helper does no
  arithmetic on them.

## Digest

Like `Trie.md`, the forest is canonical: two forests holding
the same `(AttId → AttentionRecord)` multiset have the same
root hash. `S-05/forest/digest()` is defined as just the
forest's root hash and is what scheduler checkpoints reference.

## Self-test vector

When `ignis0` grows host-language support for replaying the
attention forest (post `v0.2.5-ignis0-store`), the following
must hold:

1. `atomic_split` followed by `deduct` on the child followed by
   `dissolve_subtree_postorder` of the parent leaves the forest
   in the same state as `dissolve_subtree_postorder` of the
   parent alone, modulo the `Dissolved` entries returned.
2. `dissolve_subtree_postorder` of a subtree of size *k*
   returns exactly *k* `AttId`s in postorder.
3. Replaying the sequence of `Split`/`Deduct`/`Dissolve` weave
   entries against an empty forest produces the same root hash
   as the original forest at any checkpoint.
4. `budget_sum` at the root equals the sum of
   `budget_remaining` over all live records.

## Status

This document is the v0.2.0-helpers layout reference for the
forest operations. It is sufficient for `S-05/forest/*` helpers
to be encoded against. Encoding depends on `Trie.md` already
existing (the HAMT primitives are shared).

Open issues this spec deliberately does *not* settle:

- **Forest-wide queries.** There is intentionally no `list_minds`,
  no `iterate_attentions`. The absence of enumeration mirrors
  `S-03`'s I10 stance: the scheduler walks the attention forest
  via known `AttId`s, never by enumeration.
- **Live re-parenting.** No helper moves a node from one parent
  to another. If S-05 ever needs that, it will be a re-synthesis
  act adding a new helper, not a modification of these layouts.
