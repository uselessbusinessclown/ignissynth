# Application-level record schemas

> Sister document to `kernel/types/SCHEMA.md`. `SCHEMA.md` pins the
> low-level substance layouts the schema-helper Forms project out of
> (`WeaveEntry/v1`, `CapEntry/v1`, `AttentionRecord/v1`, `Vec/v1`,
> `Bytes/v1`). This document pins the *application-level* records —
> the typed values the upper seed Forms (S-06 through S-11) construct,
> seal, and parse back from canonical wire bytes.
>
> It exists for one concrete reason: the six wire **parsers** named in
> `kernel/forms/helpers/STUBS.md` —
> `S-06/parse_intent`, `S-07/parse_exec_state`, `S-08/parse_proof`,
> `S-08/parse_claim`, `S-09/parse_provocation`, `S-11/parse_surface` —
> cannot be encoded against an unwritten layout. `parser.form` already
> demonstrates the move for `Form/v1` → `ParsedForm/v1`; each parser in
> this document is the same recursive-descent shape against the layout
> pinned here.
>
> Every layout below is **grounded**, not invented. Each field is
> traceable to one of:
> 1. an **encoded projection** (`kernel/forms/helpers/s0*-projections.form`)
>    — the seed already commits to reading that field, so it exists;
> 2. a **breakdown record sketch** (`breakdown/S-0*.md` § Candidate A);
> 3. a **form construction body** (e.g. `S-07-form-runtime.form`'s
>    `MAKEVEC 8; SEAL ExecState`, which fixes the field order).
>
> Where the breakdown's narrative *Provocation block* and the encoded
> projections disagree on a record's field set, the disagreement is
> reconciled explicitly in that record's § Reconciliation note rather
> than silently picking one. A change to any layout below is a
> synthesis act subject to I9 (every change to type bytes requires a
> re-seal).

## Design principles

Inherited verbatim from `SCHEMA.md`:

1. **Self-describing.** Every record carries its type tag as the
   first field; the parser/projection verifies the tag and traps
   `ETYPE` on mismatch.
2. **Fixed offsets.** Within a versioned tag, every field has a fixed
   byte offset reachable by constant-time arithmetic.
3. **Hash references, not values.** Sub-substances — including
   variable-length sequences — are referenced by `Hash`, never
   embedded. A `Vec{T}` field is the 32-byte hash of a sealed
   `Vec/v1` substance (per `SCHEMA.md` § Vec{T}), *not* an inline
   array. This is what keeps every application record fixed-width and
   lets the parser build it with a single `MAKEVEC n; SEAL Tag`, the
   same way `parser.form` seals `ParsedForm` with `MAKEVEC 7`.
4. **`BOTTOM_HASH` for absent optionals.** A nullable hash field
   (e.g. `Intent.parent` at the root of an intent tree) carries
   `BOTTOM_HASH`, never a zero-length sentinel.

Because principle 3 makes every field fixed-width (a `Hash(32)` or a
`Nat`), these records have **no variable-length tail at all** — unlike
`WeaveEntry/v1`, whose `outputs[]`/`grounding[]` are inline. The
parser reads a fixed number of fixed-width fields, in canonical
(lexicographic-by-name) order, exactly as `parse_form` does.

`Nat` widths follow the `SCHEMA.md` convention: `Nat(8)` for budgets
and counters that can grow large, `Nat(4)` for small bounded counts
and program counters.

## Record index

| Record            | Type tag             | Parser                  | Producing Form |
|-------------------|----------------------|-------------------------|----------------|
| `Intent`          | `"Intent/v1"`        | `S-06/parse_intent`     | S-06, S-11     |
| `MatchResult`     | `"MatchResult/v1"`   | — (internal)            | S-06           |
| `ExecState`       | `"ExecState/v1"`     | `S-07/parse_exec_state` | S-07           |
| `Claim`           | `"Claim/v1"`         | `S-08/parse_claim`      | S-08, S-09     |
| `Proof`           | `"Proof/v1"`         | `S-08/parse_proof`      | S-08, S-09     |
| `ProofNode`       | `"ProofNode/v1"`     | — (walked by S-08)      | S-08, S-09     |
| `RuleSpec`        | `"RuleSpec/v1"`      | — (table entry)         | S-08           |
| `Provocation`     | `"Provocation/v1"`   | `S-09/parse_provocation`| S-09           |
| `BridgeRequest`   | `"BridgeRequest/v1"` | `S-11/parse_surface`    | S-11           |

`MatchResult`, `ProofNode`, and `RuleSpec` have no dedicated wire
parser — they are produced internally (S-06's matcher seals a
`MatchResult`; S-08's checker walks a tree of `ProofNode`s; `RuleSpec`
is a fixed-table entry). Their layouts are pinned here anyway because
their **projections are already encoded** and the projections fix the
offsets.

### Parser totality contracts (read off the call sites)

A parser's behavior on malformed input is **not** uniform — it is
fixed by what its caller does with the result. The contracts below are
read directly off each calling Form (not assumed), and each parser is
encoded to exactly its contract. Getting this wrong is a soundness bug:
a parser that traps where its caller expects `BOTTOM` would crash a
synthesis act that the design intends to *reject gracefully*.

| Parser                  | On malformed input | Caller's handling (verified at the call site)                         |
|-------------------------|--------------------|-----------------------------------------------------------------------|
| `S-06/parse_intent`     | return `BOTTOM_HASH`| S-06 `JMPZ` on `== BOTTOM_HASH` → its own `EILLFORMED` trap            |
| `S-11/parse_surface`    | return `BOTTOM_HASH`| S-11 `JMPZ` on `== BOTTOM_HASH` → seals a `Rejected{EILLFORMED}` Receipt|
| `S-08/parse_proof`      | return `BOTTOM_HASH`| S-08 is total (`declared-traps ()`) → wraps into a `Reject` result     |
| `S-08/parse_claim`      | return `BOTTOM_HASH`| S-08 is total (`declared-traps ()`) → wraps into a `Reject` result     |
| `S-07/parse_exec_state` | **trap `ETYPE`**   | S-07 `resume` consumes the result directly, no `BOTTOM` check (trusted)|
| `S-09/parse_provocation`| **trap `ETYPE`**   | S-09 consumes the result directly; the empty-`author` (`EANONYMOUS`) check is a *separate* semantic check on a well-formed record |

The four total parsers never trap: they validate the fixed layout and
return `BOTTOM_HASH` on any deviation. The two trapping parsers trap
`ETYPE` (the only kind a parser may emit — it is in the closed
enumeration; no new trap kind is introduced). Either way the parser is
a **total function over its input bytes** in the operational sense —
it always terminates with a defined result (a record hash, `BOTTOM`, or
a single well-defined trap). This is the same total-function discipline
the treap helpers follow (`Treap.md` § contract note).

---

## `Intent`

A sealed request: *what is wanted*, separated from *who does it* (A4.3).
Produced by S-06's caller and by S-11's bridge; consumed by
S-06/`match`. Breakdown sketch (`breakdown/S-06-intent-match.md`
§ Candidate A): `Intent { goal, inputs[], constraints[], budget,
acceptance_form_hash, parent }`.

Type tag `"Intent/v1"`.

| Field             | Offset | Type       | Meaning                                         |
|-------------------|--------|------------|-------------------------------------------------|
| `type_tag`        | 0      | `Bytes(9)` | literal `"Intent/v1"`                           |
| `budget`          | 9      | `Nat(8)`   | attention quanta the intent may consume         |
| `acceptance_form` | 17     | `Hash(32)` | Form hash whose acceptance defines success (A4.5)|
| `goal`            | 49     | `Hash(32)` | goal term/substance hash                        |
| `parent`          | 81     | `Hash(32)` | parent IntentId, or `BOTTOM_HASH` at a root     |
| `inputs`          | 113    | `Hash(32)` | sealed `Vec{Hash}` of input substance hashes    |
| `constraints`     | 145    | `Hash(32)` | sealed `Vec{Hash}` of constraint substance hashes|

Total size: 177 bytes.

Projections (`kernel/forms/helpers/s06-projections.form`):
`S-06/proj/intent_budget` reads `budget` @9; `S-06/proj/acceptance_form`
reads `acceptance_form` @17. Both offsets are fixed above.

**Reconciliation.** The breakdown names the acceptance field
`acceptance_form_hash`; the encoded projection names it
`acceptance_form`. They are the same field — a `Hash` to the acceptance
Form. This document uses the projection's name (the projection is the
operative artifact). **`S-06/parse_intent` is total: it returns the
`Intent` hash on success and `BOTTOM_HASH` on any malformed input — it
does not trap.** S-06's body `JMPZ`s on `result == BOTTOM_HASH` and
raises its *own* `EILLFORMED` trap (step 1 of the breakdown).
`EILLFORMED` is a caller-facing S-06 condition, not a new IL trap kind
and not something the parser emits. (This differs from `parse_form`,
which *does* trap `ETYPE`, because `parse_form`'s caller — the runtime
loader — trusts its input; `parse_intent`'s caller deliberately wants a
graceful reject. See § Parser totality contracts.)

## `MatchResult`

The matcher's verdict, shared by S-06 and S-11 (S-11 resolves it
through its own slot table; both project the same layout). Type tag
`"MatchResult/v1"`.

| Field              | Offset | Type        | Meaning                                       |
|--------------------|--------|-------------|-----------------------------------------------|
| `type_tag`         | 0      | `Bytes(14)` | literal `"MatchResult/v1"`                     |
| `kind`             | 14     | `Nat(4)`    | `MatchKind`: 0 = Matched, 1 = MatchedNone     |
| `none_reason`      | 18     | `Hash(32)`  | reason substance when `kind = MatchedNone`, else `BOTTOM_HASH` |
| `fulfiller_hash`   | 50     | `Hash(32)`  | chosen fulfiller when `kind = Matched`, else `BOTTOM_HASH` |
| `sub_attention_id` | 82     | `Hash(32)`  | sub-attention allocated for the fulfiller, or `BOTTOM_HASH` |

Total size: 114 bytes.

Projections: `MatchResult/proj/{kind,none_reason,fulfiller_hash,sub_attention_id}`
are bound under both the S-06 and S-11 namespaces
(`s06-projections.form`, `s11-projections.form`) and read the offsets
above. No wire parser: a `MatchResult` is sealed directly by the
matcher, never deserialised from a surface language.

## `ExecState`

The runtime reflection record S-07 builds at the start of every
`execute` and re-seals as a `Continuation` at every `yield`. The field
order is **fixed by the construction body** of
`S-07-form-runtime.form` (`MAKEVEC 8; SEAL ExecState`): the eight
values pushed are, in order, `form_hash`, `pc`, `locals`, `stack`,
`cap_view`, `weave_prev`, `inputs_hash`, `attention_id`. Type tag
`"ExecState/v1"`.

| Field          | Offset | Type        | Meaning                                          |
|----------------|--------|-------------|--------------------------------------------------|
| `type_tag`     | 0      | `Bytes(12)` | literal `"ExecState/v1"`                          |
| `form_hash`    | 12     | `Hash(32)`  | the Form being executed (read-only during run, I4)|
| `pc`           | 44     | `Nat(4)`    | program counter (instruction index)              |
| `locals`       | 48     | `Hash(32)`  | sealed `Vec{Hash}` of local slots                |
| `stack`        | 80     | `Hash(32)`  | sealed `Vec{Hash}` operand stack (top = element 0)|
| `cap_view`     | 112    | `Hash(32)`  | sealed `Vec{CapId}` reachable capabilities       |
| `weave_prev`   | 144    | `Hash(32)`  | weave tip at the call (for the Invoked entry)    |
| `inputs_hash`  | 176    | `Hash(32)`  | hash of the sealed inputs vec                    |
| `attention_id` | 208    | `Hash(32)`  | the attention this execution runs under          |

Total size: 240 bytes.

**`return_value` is derived, not stored.** The projection
`S-07/proj/return_value` (`s07-projections.form`) does **not** read a
stored field — there is no `return_value` field. It reads `stack` and
returns the top-of-stack element (element 0 of the `Vec`), which is the
value present when `RET` fires. The projection is total: on an empty
stack it traps `ETYPE` (no value to return), consistent with the
runtime never reaching `RET` with an empty stack in a well-formed Form.

`S-07/parse_exec_state` parses the eight stored fields above to restore
a `Continuation` on resume (step 5 of the interpreter loop). It is the
inverse of the `SEAL ExecState` in `execute`; the round-trip
`seal(parse(b)) == b` is the runtime's I7 (continuation-faithfulness)
obligation at the wire layer.

## `Claim`

The statement a proof discharges: *this Form replacement preserves
this invariant*. Sketch (`breakdown/S-08-proof-checker.md`
§ Candidate A): `Claim { invariant_id, form_hash_before,
form_hash_after, env }`. Type tag `"Claim/v1"`.

| Field              | Offset | Type       | Meaning                                       |
|--------------------|--------|------------|-----------------------------------------------|
| `type_tag`         | 0      | `Bytes(8)` | literal `"Claim/v1"`                          |
| `invariant_id`     | 8      | `Nat(4)`   | the invariant ordinal (I1..I10) being claimed |
| `form_hash_before` | 12     | `Hash(32)` | the Form before replacement                   |
| `form_hash_after`  | 44     | `Hash(32)` | the proposed replacement Form                 |
| `env`              | 76     | `Hash(32)` | sealed environment substance (the abstract model context the claim is stated in) |

Total size: 108 bytes.

`S-08/parse_claim` parses these four fields and is **total**: it
returns the `Claim` hash on success and `BOTTOM_HASH` on malformed
input (S-08 is `declared-traps ()` — the checker never traps; a parse
failure becomes a `Reject` result). The checker compares
`form_hash_before`/`form_hash_after` against the conclusion of the
proof's root `ProofNode` and rejects with `ClaimMismatch` on
disagreement (breakdown self-test 3) — a `Reject` verdict, not a trap.

## `Proof`

A sealed proof: a claim plus the root of its rule tree. Sketch
(`breakdown/S-08`): `Proof { claim_hash, rule_tree }`. Type tag
`"Proof/v1"`. (`STUBS.md` names the parser's return type `ProofTree`;
it is this `Proof` substance — the proof *is* its rule tree plus the
claim it discharges. The two names denote the same sealed substance.)

| Field        | Offset | Type       | Meaning                                  |
|--------------|--------|------------|------------------------------------------|
| `type_tag`   | 0      | `Bytes(8)` | literal `"Proof/v1"`                     |
| `claim_hash` | 8      | `Hash(32)` | the `Claim` this proof discharges        |
| `rule_tree`  | 40     | `Hash(32)` | root `ProofNode` of the derivation       |

Total size: 72 bytes.

`S-08/parse_proof` parses these two fields and is **total**: it returns
the `Proof` hash on success and `BOTTOM_HASH` on malformed input.
`check` then walks the `rule_tree` top-down (one `ProofNode` per node)
verifying each rule application. `check(proof, claim)` is **total**: it
returns `Accept` or `Reject{reason}`, never traps and never diverges
(breakdown constraint 1) — so neither it nor the parsers it calls may
trap. The walker projects fields out of each
`ProofNode` via the encoded `S-08/proj/*` helpers.

## `ProofNode`

One node of a natural-deduction derivation tree. Projections
(`s08-projections.form`): `S-08/proj/{conclusion,rule_id,premises}`.
Type tag `"ProofNode/v1"`.

| Field        | Offset | Type        | Meaning                                       |
|--------------|--------|-------------|-----------------------------------------------|
| `type_tag`   | 0      | `Bytes(12)` | literal `"ProofNode/v1"`                       |
| `conclusion` | 12     | `Hash(32)`  | the `Term` this node concludes                |
| `rule_id`    | 44     | `Hash(32)`  | the `RuleId` of the inference rule applied    |
| `premises`   | 76     | `Hash(32)`  | sealed `Vec{Hash}` of premise `ProofNode`s    |

Total size: 108 bytes.

`premises` is a hash to a sealed `Vec/v1` of child `ProofNode` hashes
(principle 3), so the walker recurses by reading the vec and descending
into each element. A leaf (axiom or assumption) has an empty premises
vec (`EMPTY_VEC`). The walker checks `len(premises) == RuleSpec.arity`
for the node's rule and rejects with `MissingPremise` otherwise
(breakdown self-test 2).

## `RuleSpec`

A fixed-table entry describing one inference rule. The rule table is
closed (Candidate A: "no tactics, no extensions"). Projections
(`s08-projections.form`): `S-08/proj/rule_arity`,
`S-08/proj/rule_conclusion_derivation`. Type tag `"RuleSpec/v1"`.

| Field                   | Offset | Type        | Meaning                                          |
|-------------------------|--------|-------------|--------------------------------------------------|
| `type_tag`              | 0      | `Bytes(11)` | literal `"RuleSpec/v1"`                           |
| `arity`                 | 11     | `Nat(4)`    | number of premises the rule requires             |
| `conclusion_derivation` | 15     | `Hash(32)`  | Form hash that derives the conclusion from the premise conclusions |

Total size: 47 bytes.

`conclusion_derivation` is the Form the walker invokes to verify that a
node's `conclusion` is exactly what the rule produces from its
premises' conclusions. A mismatch is a `Reject`, not a trap.

### `Term`

`ProofNode.conclusion` and the rule derivations range over `Term`
substances. For this batch the parsers treat a `Term` as **opaque,
compared by hash**: no parser constructs or destructures a `Term`, and
the checker's equality test is hash equality (the canonicaliser
guarantees structural canonicity, so hash equality is term equality).
A full `Term/v1` layout (the term language of `breakdown/S-08`
§ Candidate A: `Hash`, `Cell`, `Form`, `Nat`, `List<T>`, `Pair<T,U>`,
plus the abstract models) is deferred to the batch that encodes the
rule walker itself; it is not needed to parse `Claim` or `Proof`.

## `Provocation`

The seed of a synthesis act, read by S-09 at Stage 1. This is the
record with the **largest gap between its narrative sketch and its
operative field set**, reconciled explicitly below. Type tag
`"Provocation/v1"`.

| Field             | Offset | Type        | Meaning                                          |
|-------------------|--------|-------------|--------------------------------------------------|
| `type_tag`        | 0      | `Bytes(14)` | literal `"Provocation/v1"`                        |
| `meta_budget`     | 14     | `Nat(8)`    | synthesis attention budget for the act (A0.8)    |
| `author`          | 22     | `Hash(32)`  | `MindId` of the author; Stage 1 verifies non-empty|
| `generator_form`  | 54     | `Hash(32)`  | Form hash invoked at Stage 3 to produce candidates|
| `binding_name`    | 86     | `Hash(32)`  | the name → hash binding Stage 7 commits under    |
| `statement`       | 118    | `Hash(32)`  | sealed `Bytes` of the human-readable statement   |
| `observed`        | 150    | `Hash(32)`  | sealed `Vec{Hash}` of observed-substance digests |
| `constraint`      | 182    | `Hash(32)`  | sealed `Vec{Hash}` of constraint substances      |
| `seed_candidates` | 214    | `Hash(32)`  | sealed `Vec{Hash}` of candidate Forms handed in at seed time |

Total size: 246 bytes.

Projections (`s09-projections.form`): `S-09/proj/meta_budget` @14,
`S-09/proj/author` @22, `S-09/proj/generator_form` @54,
`S-09/proj/binding_name` @86 — all fixed above.

**Reconciliation (narrative block vs. runtime substance).** The
*Provocation block* printed at the top of every `breakdown/S-0*.md`
shows four fields: `author`, `statement`, `observed[]`, `constraint[]`.
That block is the **literary presentation** of a provocation — the
human-facing narrative an external synthesizer writes. The **runtime
`Provocation` substance** S-09 actually parses is a superset, because
the synthesis stages reference fields the literary block omits:

- `meta_budget` — `breakdown/S-09` § Candidate A step 1: "a budget
  declared in the provocation's `meta_budget` field"; encoded as
  `S-09/proj/meta_budget`.
- `generator_form` — step 4 (Stage 3): "invokes a generator Form whose
  hash is named in the provocation"; encoded as
  `S-09/proj/generator_form`.
- `seed_candidates` — step 4: "the trivial generator … reads the
  candidates verbatim from the provocation's `seed_candidates` field".
- `binding_name` — step (Stage 7): the name whose binding is updated on
  commit; encoded as `S-09/proj/binding_name`.

So the eight fields above are each grounded in either a projection or
an explicit breakdown step. The literary four are the subset a reader
sees; the runtime record carries the rest. **One field referenced but
deliberately not pinned here:** Stage 5 "evaluates the selection
criteria from the provocation". No projection and no named field backs
a distinct `selection_criteria`; the breakdown treats selection as
reading the `constraint` list (the criteria are stated as
constraints). Pinning a ninth field on the strength of a narrative
phrase would be fabrication, so this document folds selection criteria
into `constraint` and records the open question here rather than
inventing a field. If a future batch shows Stage 5 needs a distinct
field, adding it is an I9 synthesis act (tag bump + re-seal).

Stage 1 traps `EANONYMOUS` if `author` is empty — a caller-facing S-09
condition, not an IL trap kind, and a *semantic* check on an
already-well-formed record (S-09 projects `author` and compares it to
`EMPTY_HASH`, which is distinct from a `BOTTOM_HASH` parse failure).
`S-09/parse_provocation` itself is one of the two **trapping** parsers:
S-09 consumes its result directly with no `BOTTOM` check, so the parser
traps `ETYPE` on malformed wire bytes (see § Parser totality
contracts). This is unlike `parse_intent`/`parse_surface`, which are
total and return `BOTTOM`.

## `BridgeRequest`

The typed value the bridge deserialises from its surface language
before sealing an `Intent`. Sketch (`breakdown/S-11-bridge-proto.md`
§ Candidate A step 1): `BridgeRequest { goal_value, inputs[],
constraints[], budget, acceptance_form_hash, human_id_token }`. Type
tag `"BridgeRequest/v1"`.

| Field             | Offset | Type        | Meaning                                       |
|-------------------|--------|-------------|-----------------------------------------------|
| `type_tag`        | 0      | `Bytes(16)` | literal `"BridgeRequest/v1"`                   |
| `budget`          | 16     | `Nat(8)`    | requested attention budget                    |
| `acceptance_form` | 24     | `Hash(32)`  | acceptance Form hash; the bridge verifies it holds this cap |
| `goal_value`      | 56     | `Hash(32)`  | the request's goal substance                  |
| `human_id_token`  | 88     | `Hash(32)`  | opaque token identifying the human requester  |
| `inputs`          | 120    | `Hash(32)`  | sealed `Vec{Hash}` of input substances        |
| `constraints`     | 152    | `Hash(32)`  | sealed `Vec{Hash}` of constraint substances   |

Total size: 184 bytes.

Projection (`s11-projections.form`): `S-11/proj/acceptance_form` reads
`acceptance_form` @24.

`S-11/parse_surface` is the deserialiser of step 1: it parses the
surface-language wire bytes into a `BridgeRequest`. **It is total:** on
a request that fails the surface grammar it returns `BOTTOM_HASH` (it
does not trap). S-11's body `JMPZ`s on `result == BOTTOM_HASH` and
seals a `Receipt` with verdict `Rejected{EILLFORMED}` and **no**
`BridgeIn` entry (the request never became a substance). `EILLFORMED`
is a bridge verdict, not an IL trap kind. The `acceptance_form` not
being held by the bridge yields `Rejected{EUNHELDFORM}` *after* a
`BridgeIn` entry — that check is S-11's, performed after a successful
parse, not the parser's.

`parse_surface` is also the one parser whose input is a distinct
**surface grammar** rather than the fixed binary layout above: it is a
textual recursive descent (closer to `parse_form`'s shape) that emits a
`BridgeRequest` in the layout pinned here. The other five parsers
validate fixed-offset binary substance bytes.

## Status

This document is the v0.2.0-helpers layout reference for the six wire
parsers (`S-06/parse_intent`, `S-07/parse_exec_state`,
`S-08/parse_proof`, `S-08/parse_claim`, `S-09/parse_provocation`,
`S-11/parse_surface`) and for the `MatchResult`/`ProofNode`/`RuleSpec`
projections already encoded against these offsets. It is sufficient for
those parsers to be encoded in the next batch, following the
recursive-descent pattern `parser.form` establishes for `Form/v1`.

Deliberately deferred (not needed to encode the six parsers, and not
fabricated here):

- **`Term/v1`** — the proof term language. Opaque-by-hash for this
  batch; pinned when the rule walker is encoded.
- **`Stage1Record` … `Stage8Record`, `TrialRecord`, `Receipt`,
  `Continuation`, `SynthResult`, `VigilDeclaration`** — the synthesis
  and bridge intermediate records. Several have encoded projections
  (`Stage2Record/proj/axioms`, `Stage5Record/proj/*`) and will be
  pinned in the batch that encodes the S-09 stage machine and the S-11
  receipt assembly. `Continuation` is `ExecState` under a wrapper tag;
  it is pinned once `resume` is encoded.
- **`Provocation.selection_criteria`** — referenced narratively by
  Stage 5 but not backed by a projection or named field; folded into
  `constraint` until a batch demonstrates it needs separating (see
  § Provocation Reconciliation).
</content>
</invoke>
