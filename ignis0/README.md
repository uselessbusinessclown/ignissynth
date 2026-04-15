# ignis0 — stage-0 IL interpreter scaffold

The prototypical Rust implementation of `ignis0`, the stage-0
interpreter named by axiom A9 of IgnisSynth. Written against the
contract in `../kernel/IGNITION-BOOTSTRAP.md`.

## What this is

A compilable, testable Rust implementation that:

1. Enumerates all 34 IL opcodes from `../kernel/IL.md` and
   executes each of them with IL-defined semantics. No opcode
   returns `TrapKind::NotImplemented` from `exec::step` after
   v0.2.4; `NotImplemented` remains in the enum only as a
   scaffold-only marker (e.g. the `max_steps` budget guard)
   and the wire codec refuses to encode it.
2. Wires up the A9.3 fixed-point check harness for **all three
   levels**: direct (F on 42 → 43), one-level indirect via a
   hand-encoded micro-`S-07/execute` wrapper that `CALL`s F,
   and two-level indirect via a second wrapper that `CALL`s
   the first. Each indirect level is registered through
   `FormRegistry::register_wire` and produces a call chain
   whose observed frame depth matches the expected 2 / 3.
3. Implements the byte-exact Form wire codec (`src/wire.rs`)
   per `IL.md` § "Byte-exact wire grammar (v1)": encode +
   decode, round-trip over all 34 opcodes, 7 Value variants,
   and 11 TrapKind variants.
4. Provides a capability dispatch table (`src/capability.rs`)
   with a `CapabilityInvoker` trait, built-in GPU and inference
   caps, and env-configured registration — the stage-0 substrate
   side of `INVOKE`.
5. Uses BLAKE3 for substance hashing (matching S-03's
   content-addressing contract).
6. Provides an in-memory substance store that satisfies the
   abstract S-03 behaviour for the fixed-point check's needs.

## What this is *not*

- A production stage-0 substrate. Several opcodes return
  IL-defined traps that reflect stage-0 constraints rather
  than full behaviour: `APPEND` traps `EStale` (no weave log),
  `SPLIT` traps `EOverBudget` (no attention allocator), and
  `BINDSLOT` traps `EUnauthorised` (no kernel mutation
  capability in the stage-0 cap_view). These are spelled out
  in-code and tested; they unblock when the matching habitat
  substances land.
- A full canonical-parser load path from sealed substances.
  `PARSEFORM` validates wire bytes via `decode_form` and
  reseals them as a `ParsedForm/v1` cell, but the
  "`PARSEFORM` → `S-03.read` → decode" chain that a real
  ignition would use still goes through an explicit
  `FormRegistry::register_wire` call rather than resolving
  from the substance store. The micro-S-07 wrapper in
  `fixed_point.rs` is a stage-0 stand-in for the real
  `S-07/execute`, not a replacement for it.
- A persistent substance store. `SubstanceStore` is a
  `HashMap`; the persistent hash-array-mapped trie per
  S-03 lands in v0.2.5-ignis0-store (blocked on
  `kernel/types/Trie.md`).
- A proof checker. Step 4 of the ignition sequence
  (`IGNITION-BOOTSTRAP.md` § Step 4) is not yet implemented.
- A ten-step ignition sequence. Only Step 2 (the fixed-point
  check) has a concrete implementation.
- Signed or verified. The S-08 inspection record check is a
  stub.

## Note on the opcode count

`kernel/IL.md` § Opcodes enumerates 34 opcodes (4 stack/locals +
4 arith + 4 control + 4 structure + 4 substance + 4 capability +
2 weave + 2 attention + 2 trap + 4 reflection). The IL prose and
all cross-references now consistently say "Thirty-four". This
scaffold implements all 34.

## Running

```sh
cd ignis0
cargo test                                           # full test suite
cargo run -- fixed-point                             # A9.3 verdict
cargo run -- pretty-print ../kernel/forms/helpers/canon-normalise.form
cargo run -- version
```

The test harness exercises all three levels of the A9.3
fixed-point check (direct, one-level indirect, two-level
indirect), the byte-exact wire codec over all 34 opcodes, and
each opcode's IL-defined outcome (19 integration tests in
`tests/opcode_tests.rs`). A full `cargo test` pass is a
necessary (not sufficient) condition for faithful IL
interpretation.

## Layout

```
ignis0/
  Cargo.toml
  rust-toolchain.toml   # pinned stable channel (rustfmt + clippy)
  README.md             # this file
  src/
    lib.rs              # public API
    value.rs            # Value, Hash, TrapKind
    opcode.rs           # the 34 opcode variants
    exec.rs             # ExecState + small-step interpreter (CALL/RET)
    store.rs            # in-memory substance store (S-03 abstract)
    registry.rs         # content-addressed FormRegistry + slot store
    capability.rs       # CapabilityRegistry + builtin GPU/inference caps
    parser.rs           # line-oriented scaffold parser (pre-wire)
    wire.rs             # byte-exact wire codec
    pretty.rs           # Vec<Opcode> → scaffold source text
    fixed_point.rs      # A9.3 three-level check harness
    main.rs             # CLI
  tests/
    fixed_point_test.rs # A9.3 direct + indirect integration tests
    wire.rs             # wire codec round-trip + negative tests
    opcode_tests.rs     # per-opcode IL-outcome integration tests
```

## Relationship to IgnisSynth

This crate is not part of IgnisSynth. It is the external stage-0
substrate IgnisSynth depends on, per A9.1. It lives in this
repository for convenience during the scaffold phase; a proper
`ignis0` implementation will graduate to its own repository with
its own release cadence, test suite, and reviewers (per A9.2).

No act of this crate's development is recorded in the IgnisSynth
weave. No Form in the seed mentions this crate by name. The
interface between the two worlds is exactly the IL specification
in `../kernel/IL.md`.
