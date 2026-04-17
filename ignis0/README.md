# ignis0 — stage-0 IL interpreter scaffold

The prototypical Rust implementation of `ignis0`, the stage-0
interpreter named by axiom A9 of IgnisSynth. Written against the
contract in `../kernel/IGNITION-BOOTSTRAP.md`.

## What this is

A compilable, testable Rust implementation that:

1. Enumerates all 35 IL opcodes from `../kernel/IL.md` and
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
   decode, round-trip over all 35 opcodes, 7 Value variants,
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

`kernel/IL.md` § Opcodes enumerates 35 opcodes (4 stack/locals +
4 arith + 5 control + 4 structure + 4 substance + 4 capability +
2 weave + 2 attention + 2 trap + 4 reflection). The 34→35 bump
added `CALLI` (indirect call, opcode tag `0x22`) so that
`READSLOT + CALLI` composes into slot-based dynamic dispatch;
direct `CALL` with an immediate hash is retained for statically
known targets. The IL prose and all cross-references now
consistently say "Thirty-five". This scaffold implements all 35.

## Running

```sh
cd ignis0
cargo test                                           # full test suite
cargo run -- fixed-point                             # A9.3 verdict
cargo run -- pretty-print ../kernel/forms/helpers/canon-normalise.form
cargo run -- version
```

### Envelope (derivation-gated execution) commands

The crate also ships a small JSON-shaped *control plane* — `FormEnvelope` —
that demonstrates the principle "execution is gated by derivation and
structure, not just syntax". This is **not part of the IL**; it is a
parallel surface meant to make the gating principle exercisable
end-to-end while the spec-level apparatus (`S-08` proof checker, full
breakdown chain) matures. See [`src/envelope.rs`](src/envelope.rs) for
the data model.

```sh
cd ignis0
cargo run -- verify  path/to/form.envelope.json [--ledger DIR]
cargo run -- run     path/to/form.envelope.json [--ledger DIR]
cargo run -- explain path/to/form.envelope.json [--ledger DIR]
cargo run -- derive  path/to/parent.envelope.json --rule step --out child.envelope.json
```

Each envelope is a JSON object with `form_id`, `hash` (BLAKE3 of the
canonical encoding minus this field), `parents`, `rule`,
`proof_status` (`verified` / `deferred` / `invalid`),
`open_obligations`, `capabilities`, and a `payload`. The verifier
refuses any envelope whose hash does not match, whose parents are not
in the ledger, whose `proof_status` is `invalid`, or whose payload uses
a capability the envelope does not declare. Verified forms run in full
mode; deferred forms run in restricted mode (observable-only ops);
invalid forms are denied at the gate.

The `derive` subcommand produces a child envelope whose `form_id` and
hash are deterministic functions of the parent — re-running `derive`
with the same parent and rule yields the same child. The child is
`deferred` by default (children inherit the burden of proof).

The test harness exercises all three levels of the A9.3
fixed-point check (direct, one-level indirect, two-level
indirect), the byte-exact wire codec over all 35 opcodes, and
each opcode's IL-defined outcome (23 integration tests in
`tests/opcode_tests.rs`, including four CALLI tests that cover
direct stack-top dispatch, the `READSLOT + CALLI` idiom, and
the `ETYPE` / `EUNHELD` trap paths). A full `cargo test` pass
is a necessary (not sufficient) condition for faithful IL
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
    opcode.rs           # the 35 opcode variants
    exec.rs             # ExecState + small-step interpreter (CALL/RET)
    store.rs            # in-memory substance store (S-03 abstract)
    registry.rs         # content-addressed FormRegistry + slot store
    capability.rs       # CapabilityRegistry + builtin GPU/inference caps
    parser.rs           # line-oriented scaffold parser (pre-wire)
    wire.rs             # byte-exact wire codec
    pretty.rs           # Vec<Opcode> → scaffold source text
    fixed_point.rs      # A9.3 three-level check harness
    envelope.rs         # FormEnvelope (derivation-gating control plane)
    verify.rs           # envelope verifier + Ledger
    runner.rs           # envelope runner (Full/Restricted/Denied modes)
    derive.rs           # derive child envelopes from parents
    main.rs             # CLI
  tests/
    fixed_point_test.rs # A9.3 direct + indirect integration tests
    wire.rs             # wire codec round-trip + negative tests
    opcode_tests.rs     # per-opcode IL-outcome integration tests
    envelope_test.rs    # 3 demo scenarios for derivation-gated execution
  fuzz/
    Cargo.toml          # cargo-fuzz harness (separate crate)
    fuzz_targets/       # decode_form, parse_form_lines
    README.md           # how to run locally
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
