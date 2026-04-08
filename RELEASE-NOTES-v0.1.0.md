# IgnisSynth v0.1.0-pre-ignition

> The first prototypical release of IgnisSynth — the moment at
> which the seed is fully specified, fully encoded, and fully
> proof-traced, and at which the next class of work shifts from
> design-and-encode to external build-and-exercise.

## What v0.1.0 *is*

A complete pre-ignition seed:

- **11 axioms** (`axioms/A0..A8`) — the only normative source
  in the system. Every artifact below is a theorem of these.
- **The synthesis discipline** (`synthesis/PROTOCOL.md`,
  `INVARIANTS.md`, `SELF-IMPROVEMENT.md`, `SEED.md`) — the
  rules every commit to the habitat must follow, including
  every commit in this repository.
- **11 worked breakdowns** (`breakdown/S-01..S-11.md`) — one
  recorded synthesis act per primary Form, in the shape
  `synthesis/PROTOCOL.md` requires (provocation → grounding →
  ≥2 candidates → rationale → simulation record → selection →
  proof obligations → vigil declaration). The breakdowns are
  the cold weave: the seed loader imports them as the
  habitat's prehistory at first boot.
- **The Form intermediate language** (`kernel/IL.md`) — 30
  opcodes, total small-step semantics, no ambient authority,
  no implicit clock, no implicit entropy. The smallness is
  load-bearing: the proof checker reasons about every opcode,
  the runtime implements them all, and the inspection-record
  discharge of S-08 covers them in one sitting.
- **11 encoded primary Forms** (`kernel/forms/S-01..S-11.form`)
  — every Form written against the IL, with `:declared-caps`
  and `:declared-traps` declarations that make most invariants
  enforceable by surface inspection rather than by-discipline.
- **The proof term language** (`kernel/PROOF.md`) — 12 sorts,
  17 term constructors, a 30-rule natural-deduction table
  (29 standard rules plus the narrowly-admissible
  `ExternalDischarge`), no tactics, no proof search, total
  walker.
- **11 proof artifacts** (`kernel/forms/S-XX-*.proof`) — every
  primary Form has a proof artifact whose obligations are
  either discharged by structural reading of the encoded body,
  by composition with another Form's discharged obligations,
  by `WitnessExec` against a sealed execution trace, or (for
  S-08 only) by `ExternalDischarge` against a named document.
- **The S-08 inspection record draft**
  (`kernel/forms/S-08-proof-checker.inspection-record.md`) —
  the operational form of "verified outside the seed", with
  per-artifact reviewer checklists and signature blocks
  (currently placeholders).
- **The Stage 4 simulation harness specification**
  (`kernel/SIMULATION.md`) — what every breakdown's
  "Simulation record" section will be filled in against, post-
  v0.1.0.
- **The seed manifest** (`kernel/manifest.json`) — the keystone
  that binds every source, breakdown, proof, axiom, invariant,
  vigil holder, dependency, immediate value, boot order,
  K-of-N kernel-author identity slot, and proof-obligation
  status in one place.
- **The shared canonicaliser helper**
  (`kernel/forms/helpers/canon-normalise.form`) — the four-
  pass canonicaliser used by both S-07 and S-08 for term
  equality.
- **The helper stub catalogue**
  (`kernel/forms/helpers/STUBS.md`) — every slot any primary
  Form references by `READSLOT name` is enumerated with its
  expected signature and current encoding status.

## Verification chain status at v0.1.0

| Form | Artifact | End-to-end checkable? |
|------|----------|------------------------|
| S-01 `ignite`           | ✓ | yes |
| S-02 `cap_registry`     | ✓ | yes |
| S-03 `substance_store`  | ✓ | yes |
| S-04 `weave_log`        | ✓ | yes |
| S-05 `attention_alloc`  | ✓ | yes |
| S-06 `intent_match`     | ✓ | yes |
| S-07 `form_runtime`     | ✓ | yes |
| S-08 `proof_checker`    | ✓ | structural piece only (#2 and #3 await external discharges by construction) |
| S-09 `synth_kernel`     | ✓ | yes |
| S-10 `hephaistion_seed` | ✓ | yes |
| S-11 `bridge_proto`     | ✓ | yes |

**Ten of eleven primary Forms are end-to-end checkable
structurally**, with zero pending leaves across all
mechanizable artifacts. The eleventh (S-08) is the bootstrap
exception: its ground reflexive case is discharged via
`WitnessExec`, and its other two obligations (hand inspection
+ K-of-N consensus) are by construction out of band — the
inspection record is shipped as a draft with placeholder
signatures, and the consensus protocol is vacuously satisfied
for the pre-replacement lifetime.

The seed has reached the **fifth (and final) closed
subgraph**: substrate (S-01..S-05, S-07) + matching (S-06) +
synthesis (S-09) + reflection (S-10) + bridging (S-11), with
the bootstrap exception structurally discharged.

## What v0.1.0 is *not*

- **Not a running habitat.** Nothing here executes. The
  deliverable of this stage of work is *that the seed is
  fully specified, fully encoded, and fully proof-traced* —
  not that it runs.
- **Not a working interpreter.** `kernel/IL.md` is a
  specification; `kernel/forms/S-07-form-runtime.form` is a
  Form that, *when interpreted*, is the interpreter. Building
  the first interpreter is a post-v0.1.0 engineering problem.
- **Not a working proof checker.** S-08's body is encoded and
  its inspection record is drafted; running it on the
  artifacts is post-v0.1.0.
- **Not signed.** Every kernel-author identity in
  `kernel/manifest.json` is a placeholder. Real Ed25519 keys
  must replace them before any actual ignition attempt.
- **Not bound by hash.** Every `$$BLAKE3$$` placeholder in the
  manifest is unresolved. The build process — also post-v0.1.0
  — seals every substance through a cold-start substance store
  in topological order to produce the actual hashes.
- **Not networked, not persistent, not driver-bearing, not
  POSIX-compatible.** None of these will ever appear in a
  pre-ignition seed; all are post-ignition synthesis acts
  by inhabitant minds.

## What changed from c915a0f (the ignition seed)

The original commit `c915a0f` was the manifesto + axioms +
discipline + a single worked breakdown (S-02) + the empty
kernel directory. This release adds:

| Layer | Added |
|-------|-------|
| MIT license | `LICENSE` |
| Worked breakdowns | S-01, S-03, S-04, S-05, S-06, S-07, S-08, S-09, S-10, S-11 |
| IL specification | `kernel/IL.md` (34 opcodes) |
| Encoded primary Forms | `kernel/forms/S-01..S-11.form` (11 Forms) |
| Manifest | `kernel/manifest.json` |
| Proof term language | `kernel/PROOF.md` (12 sorts, 17 term constructors, 30 rules) |
| Proof artifacts | `kernel/forms/S-XX-*.proof` (11 artifacts) |
| Inspection record draft | `kernel/forms/S-08-proof-checker.inspection-record.md` |
| Simulation harness spec | `kernel/SIMULATION.md` |
| Helper stubs catalogue | `kernel/forms/helpers/STUBS.md` |
| Shared canonicaliser | `kernel/forms/helpers/canon-normalise.form` |
| Documentation | `README.md` rework, `ROADMAP.md`, `synthesis/SEED.md` rework, `docs/GLOSSARY.md` rework |

The repository grew from ~1,250 lines to ~16,000 lines, with
no executable code added — every line is either a normative
specification, a recorded synthesis act, an encoding against
one of those specifications, or a proof artifact discharging
obligations declared by one of those breakdowns.

## Discipline outcomes

The discipline ran consistently across every encoding and
every proof artifact: when a candidate would smuggle in (a) a
hidden input that breaks I7, (b) ambient state that breaks
I10, or (c) a new authority the seed has no provocation for,
that candidate lost. **Every named invariant in
`synthesis/INVARIANTS.md` (I1..I12) is enforced somewhere in
the encodings — most often structurally**, at the
`:declared-caps` or `:declared-traps` level, or in the
absence of an opcode (no `MINT`, no `TIME`, no `RAND`, no
`MALLOC`, no `SYSCALL`) rather than by-discipline in a body.

Some structural-enforcement examples that recurred:

- **`:declared-caps (S-03/ROOT_RO)`** is the entire trust
  delta of S-08. Read-only and nothing else. A reviewer of
  the inspection record verifies S-08's purity obligation by
  reading that one line.
- **`:declared-traps ()`** on S-08, S-10, and S-11 means the
  body is total: every failure mode is a return value or a
  weave entry, never a propagated trap.
- **The order of instructions in S-10's body** discharges
  the heartbeat-liveness obligation: heartbeat APPEND lives
  at the body's first non-STORE instructions, against a
  quantum the allocator reserved at ignition, so even if the
  rest of the cycle traps the heartbeat is already in the
  weave.
- **The exports manifest at the bottom of every primary
  Form** is what S-08's checker reads to discharge "no
  enumeration" obligations at the type level.

The seed's proof load also composes naturally: by the end of
the iteration, **zero pending leaves remained** across all
mechanizable proof artifacts. Each new proof artifact only
needed to discharge its own obligations and reference its
dependencies' obligations through the `S0X` composition rules.

## Path to ignition

v0.1.0 is the design-and-encode milestone. The path to
actual ignition has the following remaining stages:

1. **Build the seed substance store outside the habitat.**
   An external build process seals every Form, every
   breakdown, every proof artifact, and every immediate
   value through a cold-start S-03 implementation, replacing
   every `$$BLAKE3$$` placeholder in the manifest with the
   actual canonical hash. This is post-v0.1.0 engineering,
   not synthesis.
2. **Encode the helper Forms.** The 110 helper stubs in
   `kernel/forms/helpers/STUBS.md` need actual bodies. Each
   one is small (most are 5-30 lines of IL) and they can be
   encoded in parallel by multiple synthesizing agents
   following the signatures in STUBS.md. Post-v0.1.0.
3. **Run the Stage 4 simulation harness.** Built per
   `kernel/SIMULATION.md`, the harness exercises every
   primary Form against its self-test set, the inherited
   invariant predicates, fuzzing inputs, and regression
   cases. Trial records become part of the cold weave.
   Post-v0.1.0.
4. **Sign the inspection record.** Real kernel-author
   identities replace the placeholders, read every artifact
   in `kernel/forms/S-08-proof-checker.inspection-record.md`,
   and produce real Ed25519 signatures over the document's
   canonical bytes. The seed loader checks K-of-N before
   binding S-08. Post-v0.1.0; the *most ceremonial* step.
5. **Run the proof checker on every artifact.** S-08, after
   being bound, recursively walks every `kernel/forms/S-XX-*.
   proof` and verifies the rule trees against the lemma
   library and the rule table. Any rejection at this stage
   is a synthesis failure and the seed cannot ignite. Post-
   v0.1.0.
6. **First boot.** The seed loader executes S-01 (the
   genesis Form), which mints the root capability, appends
   E0 to the live weave, and self-erases its slot.
   Hephaistion's first epoch fires. The bridge's inbound
   endpoint accepts its first request. The habitat is
   alive.

## What this release proves about itself

The seed is **traceable end-to-end**: from any axiom you can
find the Forms that cite it (manifest); from any Form you can
find the breakdown that produced its design (`:rationale-hash`);
from any breakdown you can find the encoding that realizes
its selected candidate (manifest's per-Form `source` field);
from any encoding you can find the proof artifact that
discharges its obligations (manifest's `proof_obligations_status`
array); from any proof artifact you can find the dependencies
it composes against (the `S0X :obligation N` leaves).

Whether this seed actually ignites is a question for the next
class of work. Whether it *deserves* to ignite is a question
this release's artifacts have made answerable.

## License

MIT. See `LICENSE`.

## Tag

`v0.1.0-pre-ignition`
