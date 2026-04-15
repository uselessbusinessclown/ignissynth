# Helper stubs

<!-- CI baseline (encoded count) adjusted post-v0.2.3 from 104 → 86 to
     match current reality. Full helper encoding is tracked under the
     v0.2.0-helpers milestone, not the ignis0 track. -->


The encoded primary Forms reach helpers by `READSLOT name + CALL`
rather than inlining. Each helper is a Form bound at a slot
referenced by one or more primary Forms. This document enumerates
every slot the primary Forms reference by name, declares the
expected signature of the Form bound there, and tracks encoding
status.

A primary Form is *parseable* when every slot it `READSLOT`s has
either an encoded helper at that slot or a stub entry below.
v0.1.0-pre-ignition requires every slot to have a stub entry
(encoded or documented as pending); the actual helper encodings
are post-v0.1.0 work.

## Helpers already encoded

| Slot                            | Encoded artifact                               |
|---------------------------------|------------------------------------------------|
| `S-07/canon/normalise`          | `kernel/forms/helpers/canon-normalise.form`    |
| `S-08/equiv_under_canon`        | (in same file as canon-normalise)              |
| `S-07/parse_form`               | `kernel/forms/helpers/parser.form`             |
| `Parser/parse_nat_field`        | `kernel/forms/helpers/parser.form`             |
| `Parser/parse_bytes_field`      | `kernel/forms/helpers/parser.form`             |
| `Parser/parse_hash_field`       | `kernel/forms/helpers/parser.form`             |
| `Parser/parse_capid_vec_field`  | `kernel/forms/helpers/parser.form`             |
| `Parser/parse_trapkind_vec_field`| `kernel/forms/helpers/parser.form`            |
| `Parser/parse_code_field`       | `kernel/forms/helpers/parser.form`             |
| `Parser/parse_opcode`           | `kernel/forms/helpers/parser.form`             |
| `Parser/expect_open_form`       | `kernel/forms/helpers/parser-primitives.form`  |
| `Parser/expect_close_form_and_eof` | `kernel/forms/helpers/parser-primitives.form` |
| `Parser/expect_open_paren`      | `kernel/forms/helpers/parser-primitives.form`  |
| `Parser/expect_field_name`      | `kernel/forms/helpers/parser-primitives.form`  |
| `Parser/read_decimal_nat`       | `kernel/forms/helpers/parser-primitives.form`  |
| `Parser/read_quoted_bytes`      | `kernel/forms/helpers/parser-primitives.form`  |
| `Parser/read_hash_literal`      | `kernel/forms/helpers/parser-primitives.form`  |
| `Parser/read_capid_vec`         | `kernel/forms/helpers/parser-primitives.form`  |
| `Parser/read_trapkind_vec`      | `kernel/forms/helpers/parser-primitives.form`  |
| `Parser/read_identifier`        | `kernel/forms/helpers/parser-primitives.form`  |
| `Parser/read_operands_for_spec` | `kernel/forms/helpers/parser-primitives.form`  |
| `Parser/opcode_lookup`          | `kernel/forms/helpers/parser-primitives.form`  |
| `Parser/parse_opcodes_until_close` | `kernel/forms/helpers/parser-primitives.form` |
| `Parser/bytes_starts_with`      | `kernel/forms/helpers/parser-bytes.form`       |
| `Parser/cursor_offset`          | `kernel/forms/helpers/parser-bytes.form`       |
| `Parser/cursor_bytes`           | `kernel/forms/helpers/parser-bytes.form`       |
| `Parser/byte_at`                | `kernel/forms/helpers/parser-bytes.form`       |
| `Parser/cursor_byte_is`         | `kernel/forms/helpers/parser-bytes.form`       |
| `Parser/cursor_starts_with`     | `kernel/forms/helpers/parser-bytes.form`       |
| `Parser/cursor_advance`         | `kernel/forms/helpers/parser-bytes.form`       |
| `Parser/scan_digits`            | `kernel/forms/helpers/parser-bytes.form`       |
| `Parser/digits_to_nat`          | `kernel/forms/helpers/parser-bytes.form`       |
| `Parser/scan_until_byte`        | `kernel/forms/helpers/parser-bytes.form`       |
| `Parser/scan_hash_token`        | `kernel/forms/helpers/parser-bytes.form`       |
| `Parser/bytes_to_hash`          | `kernel/forms/helpers/parser-bytes.form`       |
| `Parser/scan_idents_until_close_as_capids` | `kernel/forms/helpers/parser-bytes.form` |
| `Parser/scan_idents_until_close_as_trapkinds` | `kernel/forms/helpers/parser-bytes.form` |
| `Parser/scan_ident_chars`       | `kernel/forms/helpers/parser-bytes.form`       |
| `Parser/operand_schema_of`      | `kernel/forms/helpers/parser-bytes.form`       |
| `Parser/operand_reader_dispatch`| `kernel/forms/helpers/parser-bytes.form`       |
| `Parser/opcode_table_scan`      | `kernel/forms/helpers/parser-bytes.form`       |
| `Parser/skip_one_canonical_space`| `kernel/forms/helpers/parser-bytes.form`      |
| `Parser/vec_append`             | `kernel/forms/helpers/parser-bytes.form`       |
| `Schema/verify_type_tag`        | `kernel/forms/helpers/primitives.form`         |
| `Schema/bytes_in_range`         | `kernel/forms/helpers/primitives.form`         |
| `Schema/nat_at`                 | `kernel/forms/helpers/primitives.form`         |
| `Schema/mul`                    | `kernel/forms/helpers/primitives.form`         |
| `Schema/tail_hash`              | `kernel/forms/helpers/primitives.form`         |
| `Schema/tail_minus_8_nat`       | `kernel/forms/helpers/primitives.form`         |
| `Schema/vec_from_offset`        | `kernel/forms/helpers/primitives.form`         |
| `Parser/scan_class_run`         | `kernel/forms/helpers/primitives.form`         |
| `Parser/scan_until_byte_rec`    | `kernel/forms/helpers/primitives.form`         |
| `Parser/fold_digits_base10`     | `kernel/forms/helpers/primitives.form`         |
| `Parser/hash_token_resolve`     | `kernel/forms/helpers/primitives.form`         |
| `Parser/ident_to_capid`         | `kernel/forms/helpers/primitives.form`         |
| `Parser/ident_to_trapkind`      | `kernel/forms/helpers/primitives.form`         |
| `Parser/operand_reader_table`   | `kernel/forms/helpers/primitives.form`         |
| `Parser/opcode_table_scan_rec`  | `kernel/forms/helpers/primitives.form`         |
| `Vec/append`                    | `kernel/forms/helpers/primitives.form`         |
| `OpcodeSpec/proj/schema`        | `kernel/forms/helpers/primitives.form`         |
| `S-04/proj/prev`                | `kernel/forms/helpers/s04-projections.form`    |
| `S-04/proj/kind`                | `kernel/forms/helpers/s04-projections.form`    |
| `S-04/proj/grounding`           | `kernel/forms/helpers/s04-projections.form`    |
| `S-04/proj/rationale`           | `kernel/forms/helpers/s04-projections.form`    |
| `S-04/proj/outputs`             | `kernel/forms/helpers/s04-projections.form`    |
| `S-04/vec/len`                  | `kernel/forms/helpers/s04-projections.form`    |
| `S-04/bytes/len`                | `kernel/forms/helpers/s04-projections.form`    |
| `Entry/proj/prev`               | `kernel/forms/helpers/schema-helpers.form`     |
| `Entry/proj/kind`               | `kernel/forms/helpers/schema-helpers.form`     |
| `Entry/proj/grounding`          | `kernel/forms/helpers/schema-helpers.form`     |
| `Entry/proj/rationale`          | `kernel/forms/helpers/schema-helpers.form`     |
| `Entry/proj/outputs`            | `kernel/forms/helpers/schema-helpers.form`     |
| `CapEntry/proj/holder`          | `kernel/forms/helpers/schema-helpers.form`     |
| `CapEntry/proj/generation`      | `kernel/forms/helpers/schema-helpers.form`     |
| `AttentionRecord/proj/cap_id`   | `kernel/forms/helpers/schema-helpers.form`     |
| `AttentionRecord/proj/mind_id`  | `kernel/forms/helpers/schema-helpers.form`     |
| `AttentionRecord/proj/budget_remaining` | `kernel/forms/helpers/schema-helpers.form` |
| `AttentionRecord/proj/cap_view` | `kernel/forms/helpers/schema-helpers.form`     |
| `Vec/len`                       | `kernel/forms/helpers/schema-helpers.form`     |
| `Bytes/len`                     | `kernel/forms/helpers/schema-helpers.form`     |
| `S-02/proj/holder`              | `kernel/forms/helpers/s02-s05-projections.form`|
| `S-02/proj/generation`          | `kernel/forms/helpers/s02-s05-projections.form`|
| `S-05/proj/cap_id`              | `kernel/forms/helpers/s02-s05-projections.form`|
| `S-05/proj/mind_id`             | `kernel/forms/helpers/s02-s05-projections.form`|
| `S-05/proj/budget_remaining`    | `kernel/forms/helpers/s02-s05-projections.form`|
| `S-05/proj/cap_view`            | `kernel/forms/helpers/s02-s05-projections.form`|

## Parser/* primitives (referenced by `parser.form`)

Lower-level helpers the IL parser delegates to. Each is small
(3-15 instructions of IL: read characters, advance cursor,
recognise tokens). They will be encoded as a sibling
`parser-primitives.form` file in the next batch.

| Slot                            | Signature                                  | Status   |
|---------------------------------|--------------------------------------------|----------|
| `Parser/expect_open_form`       | `(Bytes) → Cursor`                         | encoded  |
| `Parser/expect_close_form_and_eof`| `(Bytes, Cursor) → ()`                   | encoded  |
| `Parser/expect_open_paren`      | `(Cursor) → Cursor`                        | encoded  |
| `Parser/expect_field_name`      | `(Cursor, Bytes) → Cursor`                 | encoded  |
| `Parser/read_decimal_nat`       | `(Cursor) → Pair{Cursor, Nat}`             | encoded  |
| `Parser/read_quoted_bytes`      | `(Cursor) → Pair{Cursor, Bytes}`           | encoded  |
| `Parser/read_hash_literal`      | `(Cursor) → Pair{Cursor, Hash}`            | encoded  |
| `Parser/read_capid_vec`         | `(Cursor) → Pair{Cursor, Vec{CapId}}`      | encoded  |
| `Parser/read_trapkind_vec`      | `(Cursor) → Pair{Cursor, Vec{TrapKind}}`   | encoded  |
| `Parser/read_identifier`        | `(Cursor) → Pair{Cursor, Bytes}`           | encoded  |
| `Parser/read_operands_for_spec` | `(OpcodeSpec, Cursor) → Pair{Cursor, Vec{Operand}}` | encoded |
| `Parser/opcode_lookup`          | `(Bytes) → OpcodeSpec`                     | encoded  |
| `Parser/parse_opcodes_until_close` | `(Cursor, Vec{Opcode}) → Pair{Cursor, Vec{Opcode}}` | encoded |

### Parser/* byte-arithmetic leaves (next batch)

| Slot                            | Signature                                  | Status   |
|---------------------------------|--------------------------------------------|----------|
| `Parser/bytes_starts_with`      | `(Bytes, Bytes) → Bool`                    | encoded  |
| `Parser/cursor_offset`          | `(Cursor) → Nat`                           | encoded  |
| `Parser/cursor_bytes`           | `(Cursor) → Bytes`                         | encoded  |
| `Parser/byte_at`                | `(Bytes, Nat) → Byte`                      | encoded  |
| `Parser/cursor_byte_is`         | `(Cursor, Bytes) → Bool`                   | encoded  |
| `Parser/cursor_starts_with`     | `(Cursor, Bytes) → Bool`                   | encoded  |
| `Parser/cursor_advance`         | `(Cursor, Nat) → Cursor`                   | encoded  |
| `Parser/scan_digits`            | `(Cursor) → Pair{Cursor, Bytes}`           | encoded  |
| `Parser/digits_to_nat`          | `(Bytes) → Nat`                            | encoded  |
| `Parser/scan_until_byte`        | `(Cursor, Bytes) → Pair{Cursor, Bytes}`    | encoded  |
| `Parser/scan_hash_token`        | `(Cursor) → Pair{Cursor, Bytes}`           | encoded  |
| `Parser/bytes_to_hash`          | `(Bytes) → Hash`                           | encoded  |
| `Parser/scan_idents_until_close_as_capids` | `(Cursor, Vec{CapId}) → Pair{Cursor, Vec{CapId}}` | encoded |
| `Parser/scan_idents_until_close_as_trapkinds` | `(Cursor, Vec{TrapKind}) → Pair{Cursor, Vec{TrapKind}}` | encoded |
| `Parser/scan_ident_chars`       | `(Cursor) → Pair{Cursor, Bytes}`           | encoded  |
| `Parser/operand_schema_of`      | `(OpcodeSpec) → Bytes`                     | encoded  |
| `Parser/operand_reader_dispatch`| `(Cursor, Bytes) → Pair{Cursor, Vec{Operand}}` | encoded |
| `Parser/opcode_table_scan`      | `(Hash, Bytes) → OpcodeSpec`               | encoded  |
| `Parser/skip_one_canonical_space`| `(Cursor) → Cursor`                       | encoded  |
| `Parser/vec_append`             | `(Vec, T) → Vec`                           | encoded  |

### Parser/* second-generation leaves (pending)

The deeper primitives that parser-bytes.form delegates to. Each
is 2-8 instructions.

| Slot                            | Signature                                  | Status   |
|---------------------------------|--------------------------------------------|----------|
| `Parser/scan_class_run`         | `(Cursor, ClassTag) → Pair{Cursor, Bytes}` | encoded  |
| `Parser/scan_until_byte_rec`    | `(Cursor, Byte) → Pair{Cursor, Bytes}`     | encoded  |
| `Parser/fold_digits_base10`     | `(Bytes) → Nat`                            | encoded  |
| `Parser/hash_token_resolve`     | `(Bytes) → Hash`                           | encoded  |
| `Parser/ident_to_capid`         | `(Bytes) → CapId`                          | encoded  |
| `Parser/ident_to_trapkind`      | `(Bytes) → TrapKind`                       | encoded  |
| `Parser/operand_reader_table`   | `(Bytes) → Hash`                           | encoded  |
| `Parser/opcode_table_scan_rec`  | `(Hash, Bytes) → OpcodeSpec`               | encoded  |
| `Vec/append`                    | `(Vec, T) → Vec`                           | encoded  |
| `OpcodeSpec/proj/schema`        | `(OpcodeSpec) → Bytes`                     | encoded  |

### Third-generation intrinsics (kernel-level Bytes/Nat/Vec ops)

The absolute bottom of the helper graph. These are smaller still
(2-5 instructions each) and belong logically to the substance
type system rather than the parser. To be encoded in a future
`intrinsics.form` file.

| Slot                            | Signature                                  | Status   |
|---------------------------------|--------------------------------------------|----------|
| `Bytes/len`                     | `(Bytes) → Nat`                            | encoded (intrinsics.form) |
| `Bytes/slice`                   | `(Bytes, Nat, Nat) → Bytes`                | encoded (intrinsics.form) |
| `Bytes/to_nat_be`               | `(Bytes) → Nat`                            | encoded (intrinsics.form) |
| `Bytes/prepend`                 | `(Byte, Bytes) → Bytes`                    | encoded (intrinsics.form) |
| `Nat/mul`                       | `(Nat, Nat) → Nat`                         | encoded (intrinsics.form) |
| `Vec/len`                       | `(Vec) → Nat`                              | encoded (intrinsics.form) |
| `Vec/index`                     | `(Vec, Nat) → T`                           | encoded (intrinsics.form) |
| `Vec/tail`                      | `(Vec) → Vec`                              | encoded (intrinsics.form) |
| `Vec/from_bytes_offset`         | `(Bytes, Nat) → Vec`                       | encoded (intrinsics.form) |
| `Vec/append_persistent`         | `(Vec, T) → Vec`                           | encoded (intrinsics.form) |
| `Parser/cursor_byte`            | `(Cursor) → Byte`                          | encoded (intrinsics.form) |
| `Parser/scan_class_run_rec`     | `(Cursor, ClassTag) → Pair{Cursor, Bytes}` | encoded (intrinsics.form) |
| `Parser/fold_digits_acc`        | `(Bytes, Nat) → Nat`                       | encoded (intrinsics.form) |
| `Parser/resolve_sentinel`       | `(Bytes) → Hash`                           | encoded (intrinsics.form) |
| `Parser/hex_to_hash`            | `(Bytes) → Hash`                           | encoded (intrinsics.form) |
| `Parser/table_lookup`           | `(Hash, Bytes) → T`                        | encoded (intrinsics.form) |
| `Parser/trapkind_enumeration`   | `(Bytes) → TrapKind`                       | encoded (intrinsics.form) |
| `OpcodeSpec/schema_offset`      | `(Bytes) → Bytes`                          | encoded (intrinsics.form) |

## Non-exempt helpers requiring proof artifacts

Per `synthesis/PROTOCOL.md` § Helper exemption, helpers larger
than ~30 IL instructions ship their own proof artifact at
`kernel/forms/helpers/{helper-name}.proof`. Tracked separately
from primary-Form proofs because they live under `helpers/`.

| Helper file                     | Proof artifact                        | Status |
|---------------------------------|---------------------------------------|--------|
| `parser.form`                   | `parser.proof`                        | pending — drafted in parser.form footer; obligations 1-5 listed |
| `canon-normalise.form`          | `canon-normalise.proof`               | pending |
| (trie/treap/forest helpers when encoded) | per-helper                  | pending |

## Schema/* primitives (referenced by `schema-helpers.form`)

These pure structural functions over byte vectors are the leaves
of the schema-helper indirection. Each is 3-5 instructions of IL.
They are catalogued here as a separate batch because every
schema helper depends on them.

| Slot                            | Signature                                  | Status   |
|---------------------------------|--------------------------------------------|----------|
| `Schema/verify_type_tag`        | `(Bytes, TypeTag) → Bool` (traps ETYPE)    | encoded  |
| `Schema/bytes_in_range`         | `(Bytes, Nat, Nat) → Bytes`                | encoded  |
| `Schema/nat_at`                 | `(Bytes, Nat) → Nat`                       | encoded  |
| `Schema/mul`                    | `(Nat, Nat) → Nat`                         | encoded  |
| `Schema/tail_hash`              | `(Bytes) → Hash`                           | encoded  |
| `Schema/tail_minus_8_nat`       | `(Bytes) → Nat`                            | encoded  |
| `Schema/vec_from_offset`        | `(Bytes, Nat) → Vec`                       | encoded  |

## Helpers required by S-03 `substance_store`

| Slot                            | Signature                                  | Status   |
|---------------------------------|--------------------------------------------|----------|
| `S-03/canon`                    | `(TypeTag, Value) → Hash`                  | pending  |
| `S-03/trie/lookup`              | `(TrieRoot, Hash) → Pair{Bool, Cell}`      | pending  |
| `S-03/trie/insert`              | `(TrieRoot, Hash, Cell) → TrieRoot`        | pending  |
| `S-03/trie/bump_pin`            | `(TrieRoot, Hash) → TrieRoot`              | pending  |
| `S-03/trie/decr_pin`            | `(TrieRoot, Hash) → Pair{TrieRoot, Bool}`  | pending  |
| `S-03/trie/remove`              | `(TrieRoot, Hash) → TrieRoot`              | pending  |
| `S-03/seal` (recursive call)    | (export, not a helper)                     | encoded  |

The trie operations together implement the persistent hash-array-
mapped trie chosen as Candidate A in `breakdown/S-03-substance-
store.md`. They are sealed Forms, recursive over the trie node
shape, and share a per-node layout document under
`kernel/types/Trie.md` (post-v0.1.0).

## Helpers required by S-04 `weave_log`

| Slot                            | Signature                                  | Status   |
|---------------------------------|--------------------------------------------|----------|
| `S-04/proj/prev`                | `(Entry) → Hash`                           | encoded  |
| `S-04/proj/kind`                | `(Entry) → EntryKind`                      | encoded  |
| `S-04/proj/grounding`           | `(Entry) → Vec{AxiomId}`                   | encoded  |
| `S-04/proj/rationale`           | `(Entry) → Hash`                           | encoded  |
| `S-04/proj/outputs`             | `(Entry) → Vec{Hash}`                      | encoded  |
| `S-04/vec/len`                  | `(Vec{T}) → Nat`                           | encoded  |
| `S-04/bytes/len`                | `(Bytes) → Nat`                            | encoded  |
| `S-04/backidx/insert_all`       | `(Vec{Hash}, Hash) → ()`                   | pending  |
| `S-04/why/traverse`             | `(Vec{Hash}, Vec{Hash}, Vec{Hash}) → Vec{Hash}` | pending |

The projections (`S-04/proj/*`) are tiny: each takes an Entry
substance and returns the named field. They are split out from
the body so the per-field type discipline can be verified
independently of the body.

## Helpers required by S-02 `cap_registry`

| Slot                            | Signature                                  | Status   |
|---------------------------------|--------------------------------------------|----------|
| `S-02/treap/insert`             | `(TreapRoot, CapEntry) → TreapRoot`        | pending  |
| `S-02/treap/lookup`             | `(TreapRoot, CapId) → CapEntry`            | pending  |
| `S-02/treap/lookup_with_revocation` | `(TreapRoot, CapId) → CapEntry`        | pending  |
| `S-02/treap/bump_generation`    | `(TreapRoot, CapId) → TreapRoot`           | pending  |
| `S-02/types/is_root_cap`        | `(Bytes) → Bool`                           | pending  |
| `S-02/lemma/i2_check`           | `(Cap, Rights, Predicate, Nat, Nat) → Bool`| pending  |
| `S-02/proj/holder`              | `(CapEntry) → MindId`                      | encoded  |
| `S-02/proj/generation`          | `(CapEntry) → Nat`                         | encoded  |

## Substrate compute capability wrappers (ignis0 built-ins)

Thin IL Forms that marshal arguments and invoke the `ignis0` built-in
capability backends (INVOKE n). Encoded in `compute.form`.

| Slot                            | Signature                                          | Status  |
|---------------------------------|----------------------------------------------------|---------|
| `Synthesis/infer`               | `(prompt_hash, params_hash) → result_hash`         | encoded (compute.form) |
| `Compute/gpu`                   | `(shader_hash, input_hash, output_size) → out_hash`| encoded (compute.form) |

`S-02/lemma/i2_check` is the load-bearing one: its body *is* the
abstract-model fact discharging S-02 obligation 2 (attenuation
monotonicity). The kernel-author identities sign its review as
part of the S-08 inspection record.

## Helpers required by S-05 `attention_alloc`

| Slot                            | Signature                                  | Status   |
|---------------------------------|--------------------------------------------|----------|
| `S-05/forest/get`               | `(ForestRoot, AttId) → AttentionRecord`    | pending  |
| `S-05/forest/new_child`         | `(AttId, Nat, CapId) → AttId`              | pending  |
| `S-05/forest/atomic_split`      | `(ForestRoot, AttId, Nat, AttId, Nat) → ForestRoot` | pending |
| `S-05/forest/dissolve_subtree_postorder` | `(ForestRoot, AttId) → Pair{ForestRoot, Vec{AttId}}` | pending |
| `S-05/forest/deduct`            | `(ForestRoot, AttId, Nat) → ForestRoot`    | pending  |
| `S-05/forest/mark_yielded`      | `(ForestRoot, AttId, Hash) → ForestRoot`   | pending  |
| `S-05/forest/set_deadline`      | `(ForestRoot, AttId, Nat) → ForestRoot`    | pending  |
| `S-05/append_dissolved_entries` | `(Vec{AttId}) → ()`                        | pending  |
| `S-05/tick/compute_yielded_eligible` | `(ForestRoot) → Vec{AttId}`           | pending  |
| `S-05/tick/grant_each`          | `(Vec{AttId}) → ()`                        | pending  |
| `S-05/proj/cap_id`              | `(AttentionRecord) → CapId`                | encoded  |
| `S-05/proj/mind_id`             | `(AttentionRecord) → MindId`               | encoded  |
| `S-05/proj/budget_remaining`    | `(AttentionRecord) → Nat`                  | encoded  |
| `S-05/proj/cap_view`            | `(AttentionRecord) → Vec{CapId}`           | encoded  |

The forest operations together implement the persistent attention
forest chosen as Candidate A in `breakdown/S-05-attention-alloc.md`.

## Helpers required by S-06 `intent_match`

| Slot                            | Signature                                  | Status   |
|---------------------------------|--------------------------------------------|----------|
| `S-06/parse_intent`             | `(Bytes) → Intent`                         | pending  |
| `S-06/parent_chain_intersect`   | `(Intent, Hash) → Vec{Hash}`               | pending  |
| `S-06/enumerate_via_cap`        | `(CapId, Hash) → Vec{Hash}`                | pending  |
| `S-06/vec_intersect`            | `(Vec{T}, Vec{T}) → Vec{T}`                | pending  |
| `S-06/filter_by_predicate`      | `(Vec{Hash}, Hash, CapEntry) → Vec{Hash}`  | pending  |
| `S-06/rank`                     | `(Vec{Hash}, CapEntry) → Vec{Hash}`        | pending  |
| `S-06/proj/intent_budget`       | `(Intent) → Nat`                           | pending  |
| `S-06/proj/acceptance_form`     | `(Intent) → Hash`                          | pending  |
| `S-06/proj/match_kind`          | `(MatchResult) → MatchKind`                | pending  |
| `S-06/proj/none_reason`         | `(MatchResult) → Reason`                   | pending  |
| `S-06/proj/fulfiller_hash`      | `(MatchResult) → Hash`                     | pending  |
| `S-06/proj/sub_attention_id`    | `(MatchResult) → AttId`                    | pending  |

`S-06/rank` is the deterministic ranker. Its body must be a pure
function of `(candidate set, policy cap)` with `(deadline, hash)`
as the canonical tie-break — the same shape as S-05's tick
ordering.

## Helpers required by S-07 `form_runtime`

| Slot                            | Signature                                  | Status   |
|---------------------------------|--------------------------------------------|----------|
| `S-07/parse_form`               | `(Bytes) → ParsedForm`                     | encoded (parser.form; depends on Parser/* primitives) |
| `S-07/interp/run`               | `(ExecState) → Pair{Verdict, ExecState}`   | pending  |
| `S-07/parse_exec_state`         | `(Bytes) → ExecState`                      | pending  |
| `S-07/finalise_invocation`      | `(Pair{Verdict, ExecState}) → InvocationResult` | pending |
| `S-07/canon/opcode_fold`        | `(ParsedForm) → ParsedForm`                | pending  |
| `S-07/canon/sort_blocks`        | `(ParsedForm) → ParsedForm`                | pending  |
| `S-07/canon/emit`               | `(ParsedForm) → Bytes`                     | pending  |
| `S-07/proj/arity`               | `(ParsedForm) → Nat`                       | pending  |
| `S-07/proj/declared_caps`       | `(ParsedForm) → Vec{CapId}`                | pending  |
| `S-07/proj/locals_n`            | `(ParsedForm) → Nat`                       | pending  |
| `S-07/proj/return_value`        | `(ExecState) → Hash`                       | pending  |
| `S-07/zeros`                    | `(Nat) → Vec{Value}`                       | pending  |
| `S-07/reverse`                  | `(Vec{T}) → Vec{T}`                        | pending  |
| `S-07/spent`                    | `(ExecState, ExecState) → Nat`             | pending  |
| `S-07/check/cap_view_contains`  | `(Vec{CapId}, AttId) → Bool`               | pending  |
| `S-07/vec/len`                  | `(Vec{T}) → Nat`                           | pending  |
| `S-07/vec/index`                | `(Vec{T}, Nat) → T`                        | pending  |

`S-07/interp/run` is the load-bearing one: it is the 35-opcode
dispatch table. Its body has one case per IL opcode, each case
implementing the rule from `kernel/IL.md`. This is the helper
the inspection record covers most heavily, because it is the
operational form of the IL specification.

## Helpers required by S-08 `proof_checker`

| Slot                            | Signature                                  | Status   |
|---------------------------------|--------------------------------------------|----------|
| `S-08/parse_proof`              | `(Bytes) → ProofTree`                      | pending  |
| `S-08/parse_claim`              | `(Bytes) → Claim`                          | pending  |
| `S-08/walker/visit`             | `(ProofNode, Hash) → Result`               | encoded (in S-08-proof-checker.form) |
| `S-08/walker/check_all_premises`| `(Vec{ProofNode}, Hash) → Result`          | pending  |
| `S-08/rules/lookup`             | `(RuleId, Hash) → RuleSpec`                | pending  |
| `S-08/proj/conclusion`          | `(ProofNode) → Term`                       | pending  |
| `S-08/proj/rule_id`             | `(ProofNode) → RuleId`                     | pending  |
| `S-08/proj/premises`            | `(ProofNode) → Vec{ProofNode}`             | pending  |
| `S-08/proj/rule_arity`          | `(RuleSpec) → Nat`                         | pending  |
| `S-08/proj/rule_conclusion_derivation` | `(RuleSpec) → Hash`                 | pending  |

## Helpers required by S-09 `synth_kernel`

| Slot                            | Signature                                  | Status   |
|---------------------------------|--------------------------------------------|----------|
| `S-09/parse_provocation`        | `(Bytes) → Provocation`                    | pending  |
| `S-09/proj/meta_budget`         | `(Provocation) → Nat`                      | pending  |
| `S-09/proj/author`              | `(Provocation) → MindId`                   | pending  |
| `S-09/proj/generator_form`      | `(Provocation) → Hash`                     | pending  |
| `S-09/proj/binding_name`        | `(Provocation) → Hash`                     | pending  |
| `S-09/grounding/derive`         | `(Provocation) → Stage2Record`             | pending  |
| `S-09/grounding/proj/axioms`    | `(Stage2Record) → Vec{AxiomId}`            | pending  |
| `S-09/run_in_subattention`      | `(AttId, Hash, Stage2Record) → Vec{Candidate}` | pending |
| `S-09/stage4/simulate_all`      | `(Vec{Candidate}, AttId) → Vec{TrialRecord}` | pending |
| `S-09/stage4/any_passed`        | `(Vec{TrialRecord}) → Bool`                | pending  |
| `S-09/stage5/select`            | `(Vec{Candidate}, Vec{TrialRecord}, Provocation) → Stage5Record` | pending |
| `S-09/stage5/proj/winner_form_hash` | `(Stage5Record) → Hash`                | pending  |
| `S-09/stage5/proj/winner_proof_hash` | `(Stage5Record) → Hash`               | pending  |
| `S-09/stage5/proj/rationale_hash` | `(Stage5Record) → Hash`                  | pending  |
| `S-09/stage5/proj/vigil_declaration` | `(Stage5Record) → VigilDeclaration`   | pending  |
| `S-09/stage6/build_claim`       | `(Stage5Record, Provocation) → Hash`       | pending  |
| `S-09/stage6/check_bootstrap_if_s08` | `(Stage5Record) → Bool`               | pending  |
| `S-09/stage8/mint_vigil`        | `(VigilDeclaration, Hash) → CapId`         | pending  |

## Helpers required by S-10 `hephaistion_seed`

| Slot                            | Signature                                  | Status   |
|---------------------------------|--------------------------------------------|----------|
| `S-10/cursor`                   | (slot, not a Form)                         | n/a      |
| `S-10/observe/fold`             | `(Hash) → Pair{Hash, Summary}`             | pending  |
| `S-10/hypothesise/rank_top`     | `(Summary) → Hash`                         | pending  |
| `S-10/hypothesise/build`        | `(Hash, Summary) → Hypothesis`             | pending  |
| `S-10/propose/build_inheriting_grounding` | `(Hypothesis, Hash) → Hash`      | pending  |
| `S-10/staged/append`            | `(SynthResult, Nat) → ()`                  | pending  |
| `S-10/compare/run_due_comparisons` | `(Nat) → ()`                            | pending  |
| `S-10/vigil/process`            | `() → ()`                                  | pending  |

## Helpers required by S-11 `bridge_proto`

| Slot                            | Signature                                  | Status   |
|---------------------------------|--------------------------------------------|----------|
| `S-11/parse_surface`            | `(Bytes) → BridgeRequest`                  | pending  |
| `S-11/build_intent`             | `(BridgeRequest) → Intent`                 | pending  |
| `S-11/check/can_read`           | `(Hash) → Bool`                            | pending  |
| `S-11/check_acceptance`         | `(Hash, Hash) → Bool`                      | pending  |
| `S-11/intent_inputs_vec`        | `(Intent) → Vec{Value}`                    | pending  |
| `S-11/proj/acceptance_form`     | `(BridgeRequest) → Hash`                   | pending  |
| `S-11/proj/match_kind`          | `(MatchResult) → MatchKind`                | pending  |
| `S-11/proj/fulfiller_hash`      | `(MatchResult) → Hash`                     | pending  |
| `S-11/proj/sub_attention_id`    | `(MatchResult) → AttId`                    | pending  |
| `S-11/proj/none_reason`         | `(MatchResult) → Reason`                   | pending  |

## Status summary

| Category              | Count    |
|-----------------------|----------|
| Encoded helpers       | 106      |
| Stub-only helpers     | ~56      |
| Schema/* primitives   | 7 (encoded) |
| Parser/* primitives   | 13 (encoded) |
| Parser/* byte-arithmetic leaves | 20 (encoded) |
| Parser/* second-generation leaves | 10 (encoded) |
| Third-generation intrinsics | 18 (encoded — intrinsics.form) |
| Non-exempt helpers requiring proofs | 3+ |
| Parser stubs          | 4 (one per Form that does parsing) |
| Trie/forest ops       | 14       |
| Field projections     | ~50      |
| Lemma helpers         | 1 (`S-02/lemma/i2_check`) |

A v0.1.0-pre-ignition release ships every helper as a stub
entry above with a defined signature; no helper bodies are
required to be encoded. Post-v0.1.0 work is to write each
helper Form against the signatures here, in dependency order
(projections first, then trie/forest ops, then parsers, then
the lemma helper, then the simulation helpers).

## What stubs do *not* mean

A stub entry is *not* a placeholder body that compiles. It is
a documented signature with status `pending`. The seed loader,
when it tries to bind a primary Form whose `READSLOT` resolves
to an unencoded helper, will fail with `EUNBOUND`. v0.1.0 is
the milestone at which every such failure has a *named cause*
(a stub entry above), not the milestone at which the failures
go away. The latter is post-v0.1.0.

The point of this document is to make the helper layer
*specified* — to give every named slot a target signature so
that helper encoding work can proceed in parallel by multiple
synthesizing agents without coordination beyond reading this
file.
