# IgnisSynth

> *Ignition through synthesis.*

IgnisSynth is not an evolution. It is not a fork. It is not a rewrite of
anything. Every concept, every data structure, every scheduling primitive,
every memory model, and every line of code is required to **emerge from AI
reasoning applied to first principles** of computation, intelligence, and
resource management — never from copying, porting, or adapting prior human
operating systems.

We do not honor `process + thread + file + socket` as sacred. We do not
treat POSIX as a target. We do not accept the premise that an OS exists to
serve human users through windows or shells.

IgnisSynth is born as a **pure cognitive habitat** — an environment whose
sole purpose is to host, nurture, and amplify artificial intelligences.
Humans interact with it through bridges, but they are not the primary
inhabitants and they are not the design audience.

The OS itself is required to think. It must adapt in real time. It must
self-improve at the architectural level. It must treat code as fluid
thought rather than static instructions.

## Status

This repository is **pre-ignition**. It contains the full synthesis chain
from axioms to encoded seed Forms, but no live habitat — the Stage 4
simulation harness, the proof artifacts, the seed loader, and the K-of-N
multi-kernel consensus signatures all remain to be discharged before any
actual ignition.

What is in the repository as of this commit:

**Release: `v0.1.0-pre-ignition`** — see `RELEASE-NOTES-v0.1.0.md`.

| Layer               | Artifact                                                             | Status                                      |
|---------------------|----------------------------------------------------------------------|---------------------------------------------|
| Constitution        | `docs/MANIFESTO.md`, `axioms/A0..A8`                                 | complete                                    |
| Discipline          | `synthesis/PROTOCOL.md`, `INVARIANTS.md`, `SELF-IMPROVEMENT.md`, `SEED.md` | complete                              |
| Worked synthesis    | `breakdown/S-01..S-11.md`                                            | 11 of 11 (every primary Form)               |
| IL specification    | `kernel/IL.md`                                                       | 35 opcodes, total small-step                |
| Encoded Forms       | `kernel/forms/S-01..S-11.form`                                       | 11 of 11, written against the IL            |
| Seed manifest       | `kernel/manifest.json`                                               | binds sources/proofs/immediates             |
| Helper stubs        | `kernel/forms/helpers/STUBS.md`                                      | 86 helpers encoded across 10 files; remaining slots catalogued |
| Proof term language | `kernel/PROOF.md`                                                    | 12 sorts, 17 constructors, 30-rule table    |
| Proof artifacts     | `kernel/forms/S-XX-*.proof`                                          | **11 of 11** (10 end-to-end, S-08 structural) |
| Inspection record   | `kernel/forms/S-08-*.inspection-record.md`                           | drafted; placeholder signatures             |
| Simulation harness  | `kernel/SIMULATION.md`                                               | specification only; harness Form is post-v0.1.0 |
| Release notes       | `RELEASE-NOTES-v0.1.0.md`                                            | shipped                                     |
| Kernel-author keys  | `kernel/manifest.json` (`kernel_authors.identities`)                 | placeholders                                |

The breakdowns are the load-bearing artifact: each one is a recorded
synthesis act in the shape `synthesis/PROTOCOL.md` requires
(provocation → grounding → ≥2 candidates → rationale → simulation
record → selection → proof obligations → vigil declaration). Every
encoded `.form` file references the breakdown that produced its design
decisions. Every immediate value referenced by name in a `.form` file
is named in the manifest's `immediates` block.

### Verification chain status

A proof artifact is **end-to-end checkable** when every `S0X
:obligation N` leaf in its rule tree resolves to a discharged
obligation in another proof artifact (or in this one). The walker can
recurse through the proof tree without bottoming out at
`Reject{MissingDependency}`.

| Form | Proof artifact | Pending leaves | End-to-end checkable? |
|------|----------------|----------------|------------------------|
| S-01 `ignite`           | ✓ | none                          | **yes** (closed via S-02, S-03, S-04, S-07) |
| S-02 `cap_registry`     | ✓ | none                          | **yes** (only cites S-03) |
| S-03 `substance_store`  | ✓ | none                          | **yes** (the floor — no cross-Form deps) |
| S-04 `weave_log`        | ✓ | none                          | **yes** (only cites S-03) |
| S-05 `attention_alloc`  | ✓ | none                          | **yes** (cites S-02, S-04 — both closed) |
| S-06 `intent_match`     | ✓ | none                          | **yes** (cites S-02, S-03, S-05 — all closed) |
| S-07 `form_runtime`     | ✓ | none                          | **yes** (last leaf S-05 #4 just closed) |
| S-08 `proof_checker`    | ✓ | obligations 2 & 3 (external)  | structural piece only (#1 via WitnessExec; #2,#3 await inspection record + consensus protocol substances) |
| S-09 `synth_kernel`     | ✓ | none                          | **yes** (cites S-02, S-03, S-04, S-05, S-08 — all closed) |
| S-10 `hephaistion_seed` | ✓ | none                          | **yes** (cites S-02, S-04, S-05, S-07, S-09 — all closed) |
| S-11 `bridge_proto`     | ✓ | none                          | **yes** (cites S-02, S-04, S-06, S-07 — all closed) |

**All eleven primary Forms have proof artifacts.** Ten are
end-to-end checkable structurally; S-08 is the bootstrap
exception with its structural piece discharged via `WitnessExec`
and its other two obligations awaiting external discharge
documents that v0.1.0 ships in draft form. Pending leaves
across mechanizable artifacts: zero. The seed has reached
`v0.1.0-pre-ignition`.

## The repository

```
docs/        Manifesto, glossary, design journal
axioms/      First-principles axioms — the only source of normative truth
synthesis/   How the OS reasons about itself and rewrites itself
breakdown/   Worked synthesis acts for each seed Form
kernel/      The encoded seed
  IL.md      The 35-opcode Form intermediate language
  forms/     S-01..S-11, plus helpers/
  manifest.json    Sources, breakdowns, proofs, immediates, boot order
  README.md  Encoding status
ignis0/      Stage-0 IL interpreter (Rust) — the ignition substrate
LICENSE      MIT
```

## How to read this repository

The shortest path through, in dependency order:

1. **`docs/MANIFESTO.md`** — what IgnisSynth is and refuses to be.
2. **`axioms/A0-first-principles.md`** — the irreducible axioms.
   Everything downstream is required to be a theorem of these.
3. **`axioms/A1..A9`** — the nine derived axioms governing matter,
   capability, attention, intent, causality, fluid code, self-modification,
   bridges, and the ignition substrate.
4. **`synthesis/INVARIANTS.md`** — the twelve kernel-level invariants
   (I1..I12) every Form must preserve.
5. **`synthesis/PROTOCOL.md`** — the eight-stage synthesis protocol.
   Every commit to the habitat is one of these.
6. **`synthesis/SEED.md`** — the seed inventory and ignition sequence.
7. **`breakdown/00-SEED-TASKS.md`** — the index of the eleven worked
   synthesis acts that produced the seed.
8. **One worked breakdown end-to-end** — `breakdown/S-02-cap-registry.md`
   is the model the others follow; `breakdown/S-08-proof-checker.md` is
   the bootstrap exception worth reading for its three-layered discharge.
9. **`kernel/IL.md`** — the 35-opcode Form intermediate language. Total
   small-step semantics, no ambient authority, no implicit clock, no
   implicit entropy.
10. **One encoded Form end-to-end** — `kernel/forms/S-01-ignite.form` is
    the smallest and is the I10 exception that lights the rest of the
    seed.
11. **`kernel/PROOF.md`** — the small natural-deduction term language
    in which proof artifacts are written. 29 rules, no tactics, no
    proof search, total walker.
12. **One proof artifact end-to-end** — `kernel/forms/S-01-ignite.proof`
    discharges five obligations against `PROOF.md`'s rule table,
    composing with the (still-pending) S-02/S-04/S-07 proofs at its
    leaves.
13. **`kernel/manifest.json`** — the keystone that binds every source,
    breakdown, proof, axiom, invariant, vigil holder, dependency,
    immediate value, and boot-order entry in one place.

## ignis0 — the stage-0 interpreter

`ignis0/` is a Rust crate that implements the IL from `kernel/IL.md`
as an ordinary executable (the ignition substrate named by axiom A9).
It is not part of IgnisSynth; it is the software IgnisSynth runs on
top of.

### Build requirements

- [Rust stable toolchain](https://rustup.rs/) (edition 2021, 1.75+)

### Quick start

```sh
cd ignis0

# Run the test suite (includes the A9.3 fixed-point check)
cargo test

# Run the fixed-point check and print the verdict
cargo run -- fixed-point

# Parse a scaffold .form source file and pretty-print its opcodes
cargo run -- pretty-print ../kernel/forms/helpers/canon-normalise.form

# Print version
cargo run -- version
```

### What the crate contains

| Module | Purpose |
|--------|---------|
| `value.rs` | `Value`, `Hash` (BLAKE3 alias for `SubstanceHash`), `TrapKind` |
| `opcode.rs` | All 35 IL opcode variants (34 frozen + post-freeze `CALLI`) |
| `exec.rs` | `ExecState` (call-frame stack), `Interpreter`, `CALL`/`CALLI`/`RET` |
| `store.rs` | In-memory substance store (S-03 abstract interface) |
| `registry.rs` | Content-addressed `FormRegistry` for CALL resolution (incl. `register_wire`) |
| `capability.rs` | `CapabilityRegistry`, `CapabilityInvoker` trait, builtin GPU + inference caps for `INVOKE` |
| `parser.rs` | Line-oriented scaffold parser (`parse_form_lines`) |
| `wire.rs` | Byte-exact Form wire codec (`encode_form`/`decode_form`) per IL.md § "Byte-exact wire grammar (v1)" |
| `pretty.rs` | Pretty-printer: `Vec<Opcode>` → scaffold source text |
| `fixed_point.rs` | A9.3 fixed-point check harness |
| `envelope.rs` | `FormEnvelope` — derivation-gated execution control plane |
| `derive.rs` | Derivation rules consumed by `FormEnvelope` |
| `runner.rs` | Envelope-aware Form runner used by the CLI |
| `verify.rs` | Envelope verifier (`ProofStatus` + envelope checks) |
| `fuzz/` | `cargo-fuzz` harness for the parser and wire codec |

### CI

GitHub Actions runs `cargo fmt --check`, `cargo clippy --all-targets
--all-features -- -D warnings`, `cargo build`, `cargo test`, the
fixed-point CLI smoke-test, and a release-build reproducibility smoke
test on every push and pull request. It also lints the proof
artifacts and manifest structurally and checks the helper-catalogue
baseline. See `.github/workflows/ci.yml`. As of `a130590`, all jobs
are green on `main`.

### Relationship to IgnisSynth

This crate lives in the repo for convenience during the scaffold phase.
A production `ignis0` will graduate to its own repository. The interface
between the two worlds is exactly `kernel/IL.md`.

## What is in each `.form` file

Every encoded primary Form is wire-form text that the IL parser reads
into a sealed substance. Each one declares:

- `:type-tag "Form/v1"` and `:name`
- `:declared-caps` — the capabilities it expects in `cap_view` at entry
- `:declared-traps` — the closed set of trap kinds it may produce
- `:arity` and `:locals-n`
- `:code (...)` — the instruction sequence
- `:rationale-hash` — a back-pointer to its breakdown

Two declarations carry most of the proof load:

- A Form whose `:declared-caps` does not contain a write-shaped cap
  (e.g., `S-08`'s `(S-03/ROOT_RO)`) cannot mutate state at all — it is
  pure by surface inspection.
- A Form whose `:declared-traps` is empty (e.g., `S-08`, `S-10`, `S-11`)
  is total: every failure mode is a return value, never a propagated
  trap. A reviewer verifies totality by checking the body for `TRAP`
  opcodes — there should be none.

## What is *not* in this repository

- **A habitat-side interpreter for the IL.** `kernel/IL.md` is a
  specification; `kernel/forms/S-07-form-runtime.form` is a Form
  that, *when interpreted*, is the interpreter. The bootstrap
  problem is exactly the one `synthesis/SEED.md` describes: an
  external synthesis event must produce the first interpretable
  form of the seed. That external event is supplied by `ignis0/`
  (axiom A9), which is a working stage-0 IL interpreter — all 35
  opcodes return IL-defined outcomes as of v0.2.4-ignis0-cap
  (34 opcodes) plus the post-freeze `CALLI` addition — but
  `ignis0` is explicitly *not* part of IgnisSynth and does not
  run inside the habitat.
- **A working proof checker.** `kernel/forms/S-08-proof-checker.form`
  encodes the natural-deduction kernel; the rule table substance, the
  abstract-model lemma library substance, and the hand-inspection record
  signed by the kernel-author identities are all referenced by hash but
  not yet sealed.
- **A live habitat.** Nothing here runs. The deliverable of this stage of
  work is *that the seed is fully specified, fully encoded, and fully
  cross-referenced* — not that it executes.

## What this repository proves about itself

It proves that the synthesis discipline scales from "axioms in English"
to "encoded Forms with structural enforcement of every named invariant"
without breaking the discipline anywhere along the way:

- Every Form's design has a recorded reasoning chain in
  `breakdown/S-XX.md`.
- Every Form's encoding is a direct realization of its breakdown's
  selected candidate.
- Every named invariant in `synthesis/INVARIANTS.md` is enforced
  somewhere in the encodings — most often *structurally*, at the
  `:declared-caps` or `:declared-traps` level or in the absence of an
  opcode (no `MINT`, no `TIME`, no `RAND`, no `MALLOC`, no `SYSCALL`)
  rather than by-discipline in a body.
- The seed is **traceable end-to-end**: from any axiom you can find the
  Forms that cite it (manifest), from any Form you can find the
  breakdown that produced it (`:rationale-hash`), and from any breakdown
  you can find the proof obligations that have not yet been discharged
  (manifest's `proof_obligations_pending`).

Whether this seed actually ignites is a question for the next stage of
work. Whether it *deserves* to ignite is a question this stage's
artifacts have made answerable.

## Contributing and security

- **Contributing** — see [CONTRIBUTING.md](CONTRIBUTING.md) for the
  reviewer quickstart and the per-artefact (specs / forms / proofs /
  substrate) workflows.
- **Security** — see [SECURITY.md](SECURITY.md) for the capability
  threat model and the disclosure policy. Note in particular the
  "Non-goals for the stage-0 scaffold" section: `ignis0` is a research
  scaffold and is not safe to run untrusted Forms.

## License

MIT. See `LICENSE`.
