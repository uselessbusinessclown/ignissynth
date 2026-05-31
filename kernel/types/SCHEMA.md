# Substance schemas

This document describes the byte layout of every named substance
type the seed Forms project fields out of. The schema-helper
Forms (`Entry/proj/*`, `Vec/len`, `Bytes/len`, `CapEntry/proj/*`,
`AttentionRecord/proj/*`, etc.) each implement one row of the
tables below.

The layouts are themselves versioned by their type tag (per A1.2
— type is part of the hash). A change to any layout below
requires bumping the type tag and re-sealing every substance
that uses the old tag, which is a synthesis act subject to I9.

## Design principles

1. **Self-describing.** Every substance carries its type tag as
   the first field. The schema helper can verify the tag before
   accessing any other field, trapping `ETYPE` on mismatch.
2. **Fixed offsets.** Within a versioned tag, every field has a
   fixed byte offset. Schema helpers do constant-time arithmetic
   to reach a field; no parsing.
3. **Variable-length tail.** The last field of each layout is
   the only variable-length field. Vec and Bytes substances
   carry a length prefix immediately after the type tag, then
   the elements/bytes.
4. **Hash references, not values.** Sub-substances are
   referenced by `Hash`, never embedded. The schema helpers
   never recurse into sub-substances; recursion is the caller's
   job.

## Entry

The weave entry substance written by `S-04/append`. Type tag
`"WeaveEntry/v1"`.

| Field         | Offset | Type            | Meaning                              |
|---------------|--------|-----------------|--------------------------------------|
| `type_tag`    | 0      | `Bytes(13)`     | literal `"WeaveEntry/v1"`            |
| `kind`        | 13     | `EntryKind(8)`  | one of: Sealed, Reclaimed, Synthesized, Invoked, Trapped, Granted, Split, Dissolved, Matched, MatchedNone, BridgeIn, BridgeOut, Heartbeat, Hypothesised, RejectedHypothesis, IgnitionReplayAttempted, SynthStage |
| `prev`        | 21     | `Hash(32)`      | hash of the previous tip entry       |
| `outputs_n`   | 53     | `Nat(4)`        | number of output hashes              |
| `outputs`     | 57     | `Hash[]`        | the output substance hashes          |
| `grounding_n` | 57+32n | `Nat(4)`        | number of axiom ids (Synthesized only) |
| `grounding`   | 61+32n | `AxiomId[]`     | the axiom ids (Synthesized only)     |
| `rationale`   | tail   | `Hash(32)`      | rationale substance hash (Synthesized only) |

`S-04/proj/{prev,kind,grounding,rationale,outputs}` each read
the entry, verify the type tag, and return the field at the
given offset.

## CapEntry

The capability registry entry produced by `S-02/recognise_root`
and `S-02/attenuate`. Type tag `"CapEntry/v1"`.

| Field          | Offset | Type           | Meaning                              |
|----------------|--------|----------------|--------------------------------------|
| `type_tag`     | 0      | `Bytes(11)`    | literal `"CapEntry/v1"`              |
| `parent`       | 11     | `Hash(32)`     | parent CapId, or `BOTTOM_HASH`       |
| `rights`       | 43     | `Rights(32)`   | the rights bitmap                    |
| `predicate`    | 75     | `Hash(32)`     | predicate Form hash                  |
| `holder`       | 107    | `Hash(32)`     | mind id of the holder                |
| `generation`   | 139    | `Nat(8)`       | generation counter for revocation    |

`S-02/proj/{holder,generation}` read the entry and return the
field at the given offset.

## AttentionRecord

The forest entry produced by `S-05/split` and stored in the
forest trie. Type tag `"AttentionRecord/v1"`.

| Field             | Offset | Type            | Meaning                            |
|-------------------|--------|-----------------|------------------------------------|
| `type_tag`        | 0      | `Bytes(18)`     | literal `"AttentionRecord/v1"`     |
| `parent`          | 18     | `Hash(32)`      | parent AttId, or `BOTTOM_HASH`     |
| `mind_id`         | 50     | `Hash(32)`      | mind that owns this attention      |
| `cap_id`          | 82     | `Hash(32)`      | focus capability id                |
| `cap_view_n`      | 114    | `Nat(4)`        | number of caps in view             |
| `cap_view`        | 118    | `CapId[]`       | the cap_view itself                |
| `budget_remaining`| tail-8 | `Nat(8)`        | quanta remaining at this snapshot  |
| `deadline`        | tail   | `Nat(8)`        | deadline as weave-entry-count offset|

`S-05/proj/{cap_id,mind_id,budget_remaining,cap_view}` each
read the record, verify the type tag, and return the field at
the given offset.

## Vec{T}

A length-prefixed sequence. Type tag `"Vec/v1"`.

| Field      | Offset | Type        | Meaning                          |
|------------|--------|-------------|----------------------------------|
| `type_tag` | 0      | `Bytes(6)`  | literal `"Vec/v1"`               |
| `n`        | 6      | `Nat(4)`    | number of elements               |
| `elements` | 10     | `T[]`       | elements in order                |

`Vec/len` reads the substance, verifies the type tag, returns
the `n` field. The element width depends on `T`; the schema
helper trusts the caller to interpret the elements correctly
(the element type is itself part of `T`'s own type tag).

## Bytes

A length-prefixed byte sequence. Type tag `"Bytes/v1"`.

| Field      | Offset | Type        | Meaning                          |
|------------|--------|-------------|----------------------------------|
| `type_tag` | 0      | `Bytes(8)`  | literal `"Bytes/v1"`             |
| `n`        | 8      | `Nat(8)`    | number of bytes                  |
| `bytes`    | 16     | `Byte[]`    | the bytes themselves             |

`Bytes/len` reads the substance, verifies the type tag, returns
the `n` field.

## Companion documents

The persistent data structures the trie/treap/forest helpers walk
are specified in sister documents under `kernel/types/`:

- **`Trie.md`** — persistent HAMT for the `S-03/substance-store`
  (`TrieEmpty/v1`, `TrieBranch/v1`, `TrieLeaf/v1`, `Cell/v1`).
- **`Treap.md`** — persistent treap for the `S-02/cap-registry`
  (`TreapEmpty/v1`, `TreapBranch/v1`).
- **`Forest.md`** — persistent attention forest for `S-05/attention-alloc`
  (`ForestEmpty/v1`, `ForestBranch/v1`, `ForestLeaf/v1`).
- **`Records.md`** — application-level record layouts the upper Forms
  (S-06..S-11) construct, seal, and parse back from wire bytes
  (`Intent/v1`, `MatchResult/v1`, `ExecState/v1`, `Claim/v1`,
  `Proof/v1`, `ProofNode/v1`, `RuleSpec/v1`, `Provocation/v1`,
  `BridgeRequest/v1`). This is the layout reference the six wire
  parsers (`S-06/parse_intent`, `S-07/parse_exec_state`,
  `S-08/parse_proof`, `S-08/parse_claim`, `S-09/parse_provocation`,
  `S-11/parse_surface`) are encoded against.

The first three documents are the v0.2.0-helpers layout reference
for the `S-03/trie/*`, `S-02/treap/*`, and `S-05/forest/*` helper
families. They are also the spec `ignis0`'s `v0.2.5-ignis0-store`
milestone implements (replacing the HashMap-backed
`SubstanceStore` with a real persistent HAMT). `Records.md` is the
layout reference for the wire parsers and the upper-Form projections.

## Status

This document is the v0.2.0-helpers layout reference for the
schema helpers and the records the persistent data structures
above carry. It is sufficient for the schema helpers to be
encoded against.

The application-level records the upper Forms parse and project —
`Intent`, `MatchResult`, `ExecState`, `Claim`, `Proof`,
`ProofNode`, `RuleSpec`, `Provocation`, `BridgeRequest` — are now
pinned in the companion `Records.md` (the layout reference for the
six wire parsers). The remaining substance types (`Stage1Record`
through `Stage8Record`, `Receipt`, `Continuation`, `Term`,
`TrialRecord`, `Hypothesis`, `Verdict`) will gain layout entries in
subsequent v0.2.0 batches as the helpers that need them are
encoded. (`ParsedForm` is pinned inline in
`kernel/forms/helpers/parser.form`.)
