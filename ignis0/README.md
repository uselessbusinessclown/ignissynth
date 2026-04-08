# ignis0 — stage-0 IL interpreter scaffold

The prototypical Rust implementation of `ignis0`, the stage-0
interpreter named by axiom A9 of IgnisSynth. Written against the
contract in `../kernel/IGNITION-BOOTSTRAP.md`.

## What this is

A compilable, testable Rust scaffold that:

1. Enumerates all 34 IL opcodes from `../kernel/IL.md`.
2. Implements the direct-execution interpreter for the opcodes
   needed to run the canonical fixed-point Form `F` (`STORE`,
   `LOAD`, `PUSH`, `ADD`, `RET`).
3. Wires up the A9.3 fixed-point check harness for the direct
   case.
4. Stubs the indirect cases (one-level and two-level S-07
   interpretation) honestly — they compile, return an
   explicit `NotImplemented` verdict, and cite the specific
   pending work items.
5. Uses BLAKE3 for substance hashing (matching S-03's
   content-addressing contract).
6. Provides an in-memory substance store that satisfies the
   abstract S-03 behaviour for the fixed-point check's needs.

## What this is *not*

- A complete stage-0 implementation. Most opcodes are stubbed
  with `Trap::NotImplemented`.
- Wired up to the wire parser yet. A byte-exact codec for
  Form wire bytes lives in `src/wire.rs` and is exercised by
  `tests/wire.rs` (round-trip property tests over randomly
  generated Forms), but the interpreter still runs against
  hand-constructed or line-parsed `Vec<Opcode>` because CALL
  and a form loader are prerequisites — see v0.2.1-ignis0-call
  in `../ROADMAP.md`.
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
cargo test                 # run the fixed-point test harness
cargo run -- fixed-point   # print the fixed-point verdict
```

The test harness exercises the direct case of the A9.3
fixed-point check: construct `F`, invoke it on input `42`,
assert the result is `43`. If this passes, `ignis0` correctly
implements the five opcodes F needs, which is a necessary
(not sufficient) condition for faithful IL interpretation.

## Layout

```
ignis0/
  Cargo.toml
  README.md         # this file
  src/
    lib.rs          # public API
    value.rs        # Value, Hash, TrapKind
    opcode.rs       # the 34 opcode variants
    exec.rs         # ExecState + small-step interpreter
    store.rs        # in-memory substance store
    fixed_point.rs  # A9.3 check harness
    parser.rs       # line-oriented scaffold parser (pre-wire)
    wire.rs         # byte-exact wire codec (v0.2.2)
    main.rs         # CLI
  tests/
    fixed_point_test.rs  # A9.3 direct-case integration test
    wire.rs              # wire codec round-trip + negative tests
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
