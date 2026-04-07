# Form IL — the seed intermediate language

This document specifies the small intermediate language in which the
seed Forms are written. It is itself a synthesis artifact: the
provocation is "the breakdowns reference a Form IL that does not yet
exist; without a defined IL, no `.form` file has a meaning". The
grounding is A0.6 (code is thought, not artifact), A0.7 (synthesis
is the only way new structure enters), A6.1 (a unit of code is a
substance of executable type), A6.5 (the kernel is a Form too), and
S-07 obligation 4 + 5 (the IL must contain no instruction that
forges authority and every instruction must have a well-defined
successor, trap, or yield).

The IL is small on purpose. The smallness is the load-bearing
property: S-08's proof checker must be able to reason about every
instruction, S-07's runtime must implement them all, and humans
holding the inspection-record discharge of S-08 must be able to
read every opcode in one sitting.

## Design constraints (from the breakdowns)

1. **~30 opcodes** (S-07 budget claim).
2. **Total small-step semantics**: every instruction is one of
   `step`, `trap{kind}`, `yield{continuation}` — no undefined
   behavior (S-07 obligation 5).
3. **No forging**: no instruction returns a capability whose
   ancestor is not in the current attention's `cap_view` (S-07
   obligation 4).
4. **Pure given declared inputs**: no implicit time, no implicit
   entropy, no implicit network, no implicit clock. Anything
   non-deterministic must be a capability operation against a
   substance the attention holds (S-05's I7 boundary).
5. **Content addressed**: every value the IL operates on is either
   an immediate (a `Nat`, a `Bool`, a `Hash`) or a substance hash
   resolved through S-03. There are no pointers.
6. **Self-describing**: the IL is itself a substance type, and
   any Form encoded in the IL is a sealed value that the IL's own
   parser (a Form, written in the IL) can read.

## Value types

The IL operates over a closed set of value types. New types may
enter the system only through synthesis acts that extend the IL,
which themselves require S-09 + S-08.

| Type        | Description                                        |
|-------------|----------------------------------------------------|
| `Bool`      | `true` or `false`                                  |
| `Nat`       | unbounded non-negative integer                     |
| `Hash`      | a 32-byte BLAKE3 substance hash (per S-03)         |
| `CapId`     | a `Hash` that S-02 recognises as a capability id   |
| `Cell{T}`   | a sealed substance of type `T`, addressed by hash  |
| `Vec{T}`    | a sealed sequence of values of type `T`            |
| `Pair{T,U}` | a sealed pair                                      |
| `Cont`      | a sealed `ExecState` (S-07 continuation)           |
| `Trap{K}`   | a sealed trap value with kind `K`                  |

`Cell{T}`, `Vec{T}`, `Pair{T,U}`, and `Cont` are all stored through
S-03 and named by hash. The IL never operates on raw bytes — every
"value" the runtime touches is either an immediate or a hash with
a known type tag.

## ExecState

Every step of a Form's execution is a transition over an
`ExecState`:

```
ExecState {
    form_hash:    Hash,         // the Form being executed
    pc:           Nat,          // instruction pointer
    locals:       Vec{Value},   // named slots, indexed by Nat
    stack:        Vec{Value},   // operand stack
    cap_view:     Vec{CapId},   // the attention's reachable caps
    weave_prev:   Hash,         // tip hash at call entry
    inputs_hash:  Hash,         // sealed inputs vector
    attention_id: Hash,         // S-05 attention id this is running under
}
```

`ExecState` is itself a substance type. A `Cont` is exactly a
sealed `ExecState`.

## Opcodes

Thirty exactly. Grouped by purpose. Each line names the opcode,
its stack effect, and the small-step rule. Every rule is total:
it produces `step(s')`, `trap{kind}`, or `yield(cont)` and
nothing else.

### Stack and locals (4)

| Opcode      | Effect              | Rule                                      |
|-------------|---------------------|-------------------------------------------|
| `PUSH imm`  | `() → (v)`          | push immediate value `imm`                |
| `POP`       | `(v) → ()`          | discard top                               |
| `LOAD i`    | `() → (v)`          | push `locals[i]`; trap `EBADLOCAL` if oob |
| `STORE i`   | `(v) → ()`          | `locals[i] := v`                          |

### Arithmetic and comparison (4)

| Opcode | Effect              | Rule                                      |
|--------|---------------------|-------------------------------------------|
| `ADD`  | `(a,b) → (a+b)`     | `Nat` only; trap `ETYPE` otherwise        |
| `SUB`  | `(a,b) → (a-b)`     | trap `EUNDERFLOW` if `b > a`              |
| `EQ`   | `(a,b) → (a==b)`    | structural equality, total                |
| `LT`   | `(a,b) → (a<b)`     | `Nat` only                                |

### Control flow (4)

| Opcode      | Effect           | Rule                                       |
|-------------|------------------|--------------------------------------------|
| `JMP off`   | `() → ()`        | `pc := pc + off`                           |
| `JMPZ off`  | `(b) → ()`       | branch if `b = false`                      |
| `CALL h n`  | `(arg₁..argₙ) → (ret)` | invoke Form at `h` with `n` args via S-07 sub-call    |
| `RET`       | `(v) → ()`       | return `v` to caller; emits one `Invoked` weave entry |

### Structure (4)

| Opcode       | Effect                  | Rule                                                |
|--------------|-------------------------|-----------------------------------------------------|
| `MAKEPAIR`   | `(a,b) → (Pair{a,b})`   | seal a `Pair` through S-03                          |
| `FST`        | `(Pair{a,_}) → (a)`     | unseal first; trap `ETYPE` if not a Pair            |
| `SND`        | `(Pair{_,b}) → (b)`     | unseal second                                       |
| `MAKEVEC n`  | `(v₁..vₙ) → (Vec)`      | seal a `Vec` of length `n` through S-03             |

### Substance (4)

| Opcode    | Effect                | Rule                                                          |
|-----------|-----------------------|---------------------------------------------------------------|
| `SEAL t`  | `(v) → (Hash)`        | call `S-03.seal(t, v)`; push the resulting hash               |
| `READ`    | `(Hash) → (v)`        | call `S-03.read(h)`; trap `EUNHELD` if no Read cap on `h`     |
| `PIN`     | `(Hash) → ()`         | call `S-03.pin(h)`; trap `EUNHELD` likewise                   |
| `UNPIN`   | `(Hash) → ()`         | call `S-03.unpin(h)`; trap `EUNHELD` likewise                 |

### Capability (4)

| Opcode      | Effect                       | Rule                                                                       |
|-------------|------------------------------|----------------------------------------------------------------------------|
| `CAPHELD`   | `(CapId) → (Bool)`           | `cap_view.contains(c)`                                                     |
| `ATTENUATE` | `(CapId, predicate) → (CapId')` | call `S-02.attenuate(c, p)`; trap `ENOTHELD` if `c ∉ cap_view`          |
| `INVOKE`    | `(CapId, args…) → (result)`  | call the operation the cap names; trap `ENOTHELD` if `c ∉ cap_view`        |
| `REVOKE`    | `(CapId) → ()`               | call `S-02.revoke(c)`; trap `ENOTHELD` if `c ∉ cap_view` or `c` not minted by this attention |

There is no `MINT`. Capability creation is the I10 exception named
in S-01 and lives only inside `ignite`. Every other capability
arrives through `ATTENUATE` from a parent the attention already
holds.

### Weave (2)

| Opcode    | Effect                            | Rule                                                              |
|-----------|-----------------------------------|-------------------------------------------------------------------|
| `APPEND`  | `(EntryHash) → (TipHash)`         | call `S-04.append(e)`; trap `ESTALE` if `e.prev ≠ current_tip`    |
| `WHY`     | `(SubstanceHash) → (Vec{EntryHash})` | call `S-04.why(s)`                                             |

### Attention and yield (2)

| Opcode  | Effect                 | Rule                                                                              |
|---------|------------------------|-----------------------------------------------------------------------------------|
| `YIELD` | `() → ()`              | `seal(ExecState)`; emit a `Cont`; return control to S-05; resume from sealed `pc` |
| `SPLIT` | `(budget) → (AttId)`   | call `S-05.split(current_attention, budget)`; trap `EOVERBUDGET` if disallowed    |

### Trap (2)

| Opcode      | Effect       | Rule                                                                |
|-------------|--------------|---------------------------------------------------------------------|
| `TRAP k`    | `() → ⊥`     | append a `Trapped{form_hash, pc, kind: k}` weave entry; return to caller's frame with error |
| `ASSERT`    | `(Bool) → ()`| if `false`, behave as `TRAP EASSERT`                                |

### Reflection (4)

| Opcode      | Effect                             | Rule                                                                                            |
|-------------|------------------------------------|-------------------------------------------------------------------------------------------------|
| `SELFHASH`  | `() → (Hash)`                      | push the current `form_hash` from `ExecState` (A6.4 base case)                                  |
| `PARSEFORM` | `(Hash) → (Vec{Opcode})`           | call the IL parser Form on the substance at `h`; trap `ETYPE` if not a `Form` substance         |
| `BINDSLOT`  | `(name_hash, form_hash) → ()`      | atomically advance the binding `name_hash → form_hash`; trap `EUNAUTHORISED` if no kernel mutation cap held |
| `READSLOT`  | `(name_hash) → (form_hash)`        | look up the current binding for `name_hash`                                                      |

That is the entire IL. Thirty opcodes. Three of them
(`ATTENUATE`, `INVOKE`, `BINDSLOT`) reduce to operations on other
seed Forms, and the rest are fully defined by their rules above.

## Trap kinds

A closed enumeration. New trap kinds may enter the IL only through
synthesis acts that extend it (which the proof checker must accept
under I9).

| Kind                  | Cause                                                           |
|-----------------------|-----------------------------------------------------------------|
| `EBADLOCAL`           | `LOAD`/`STORE` index out of range                               |
| `ETYPE`               | operand type mismatch                                           |
| `EUNDERFLOW`          | `Nat` subtraction would go negative                             |
| `EUNHELD`             | substance read/pin/unpin without the required cap               |
| `ENOTHELD`            | capability operation against a `CapId` not in `cap_view`        |
| `ESTALE`              | weave append where `entry.prev ≠ current_tip`                   |
| `EOVERBUDGET`         | attention split or grant would violate I6                       |
| `EASSERT`             | `ASSERT` of `false`                                             |
| `EUNAUTHORISED`       | `BINDSLOT` without the kernel mutation capability               |
| `EIGNITED`            | `ignite` invoked after the first successful ignition            |
| `EREPLAYDIVERGED`     | non-clean continuation replayed without its declared inputs     |

## Encoding

A Form, in the wire form, is a sealed substance whose canonical
bytes are:

```
Form {
    type_tag:        "Form/v1"
    declared_caps:   Vec{CapId}      // caps the Form expects in cap_view at entry
    declared_traps:  Vec{TrapKind}   // trap kinds this Form may produce
    arity:           Nat             // expected stack depth at entry
    locals_n:        Nat             // size of locals array
    code:            Vec{Opcode}
}
```

`type_tag` is part of the hash (per A1.2 — type is intrinsic).
`Opcode` is itself a small sealed value of one of the 30 shapes
above. `declared_caps` and `declared_traps` are part of the Form's
proof obligation: a Form whose execution invokes a cap not in
`declared_caps`, or produces a trap not in `declared_traps`,
fails Stage 4 simulation regardless of test outcomes.

## What is not in the IL

To make the absences explicit:

- No `MINT` — capability creation is the I10 exception (S-01).
- No `TIME` or `NOW` — time is `weave.tip` order; see S-05.
- No `RAND` — entropy is a capability whose `INVOKE` returns a
  fresh substance; the IL has no implicit entropy.
- No `MALLOC`, `FREE`, `ALLOC`, `LOAD_RAW`, `STORE_RAW` — there
  are no pointers, no addresses, no untyped memory.
- No `SYSCALL` — every operation is either an opcode above or an
  `INVOKE` of a capability the Form already holds.
- No `IMPORT`, `LOAD_LIBRARY`, `DLOPEN` — Forms are summoned by
  `READSLOT` + `CALL`, never loaded from a path.
- No `THROW`, `CATCH` — traps are not exceptions; they are
  weave entries and they propagate to the caller's frame as
  return values, not as control transfers.

## What this IL costs

The minimum credible implementation of S-07 against this IL is
~1400 lines of Form IL itself (a small interpreter loop, the IL
canonicaliser, and the continuation seal/resume + trap handling),
which is the budget S-07 declared. The IL canonicaliser is
shared with S-08 (~400 lines), which is why S-08's budget
includes a "shared canonicaliser delta" line.

## What this IL enables

It enables every other seed Form to have a meaning. S-01 through
S-11 are written in this IL or in something equivalent up to
parser-canonicalisation. The IL itself is a Form (a parser Form
that reads the wire format above and produces an `ExecState`-
shaped value), and `PARSEFORM` is the opcode that lets a Form
read another Form's source — which is the substrate A6.4 (self-
reading is normal) requires.
