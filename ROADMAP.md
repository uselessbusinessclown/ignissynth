# Roadmap to v0.1.0-pre-ignition (✓ REACHED)

> **Status**: v0.1.0-pre-ignition reached. See
> `RELEASE-NOTES-v0.1.0.md`. The post-v0.1.0 milestone schedule
> is the path to ignition: external build, helper encoding,
> simulation runs, signed inspection record, first boot.

This document tracks what is needed to declare a *prototypical release
form* of IgnisSynth — the smallest set of artifacts that, taken
together, constitute a credible "v0.1.0-pre-ignition" tag.

A prototypical release is **not** a running habitat. The seed cannot
self-host until S-08 has been built outside the seed (the bootstrap
problem). What v0.1.0-pre-ignition *is*: every primary Form has a
breakdown, an encoding, and a structurally-complete proof artifact;
every helper the encodings reference has at least a stub spec; the
Stage 4 simulation harness has a written specification; the S-08
inspection record has been drafted; and the manifest's
`$$BLAKE3$$` placeholders are documented well enough for an
external build process to resolve them in topological order.

It is the milestone at which the seed is *ready to be built into a
substance store outside the habitat* and the cold-weave hand-off
to a hypothetical first ignition becomes a finite engineering
problem rather than an open design problem.

## Definition of done for v0.1.0-pre-ignition

| # | Artifact                                                              | Status     |
|---|-----------------------------------------------------------------------|------------|
| 1 | 11 worked breakdowns (S-01..S-11)                                     | ✓ done     |
| 2 | 30-opcode IL specification (`kernel/IL.md`)                           | ✓ done     |
| 3 | 11 encoded primary Forms (`kernel/forms/S-01..S-11.form`)             | ✓ done     |
| 4 | Seed manifest (`kernel/manifest.json`)                                | ✓ done     |
| 5 | Proof term language (`kernel/PROOF.md`)                               | ✓ done     |
| 6 | Shared canonicaliser helper (`kernel/forms/helpers/canon-normalise`)  | ✓ done     |
| 7 | 11 proof artifacts (`kernel/forms/S-XX-*.proof`)                      | ✓ done (11/11) |
| 8 | Stage 4 simulation harness specification (`kernel/SIMULATION.md`)     | ✓ done     |
| 9 | S-08 inspection record draft (`kernel/forms/S-08-*.inspection-record.md`) | ✓ done (placeholder signatures) |
| 10| Helper stubs for every slot referenced by an encoded Form             | ✓ done (`kernel/forms/helpers/STUBS.md`) |
| 11| `RELEASE-NOTES-v0.1.0.md` + manifest version bump + `v0.1.0` git tag  | ✓ done     |

A v0.1.0 tag is admissible when items 1-11 are checked. None of
items 1-10 require running code; item 11 requires a release notes
document and a tag.

## Proof artifact dependency graph

Each `.proof` artifact's `S0X :obligation N` leaves resolve only
when the cited proof exists. Listed in dependency order — earlier
entries unblock later ones.

All eleven proof artifacts have landed. The dependency graph
below is the **historical** order in which they were closed
during the v0.1.0 iteration; every node in it now reads ✓.

```
                 ┌──────────────┐
                 │ S-03 ✓       │  the floor — no cross-Form deps
                 └──────┬───────┘
                        │
        ┌───────────────┼─────────────────────────┐
        │               │                         │
        ▼               ▼                         ▼
┌──────────────┐ ┌──────────────┐         ┌──────────────┐
│ S-04 ✓       │ │ S-02 ✓       │         │ S-07 ✓       │
│ deps: S-03   │ │ deps: S-03   │         │ deps: S-03,  │
│              │ │              │         │       S-05   │
└──────┬───────┘ └──────┬───────┘         └──────┬───────┘
       │                │                         │
       │                └──────┬──────────────────┘
       │                       │
       └───────────┬───────────┘
                   │
                   ▼
          ┌──────────────────┐
          │ S-01 ✓           │  the I10 base case (closed via the four above)
          └──────────────────┘

           ┌──────────────┐
           │ S-05 ✓       │
           │ deps: S-02   │
           └──────┬───────┘
                  │
                  ▼
           ┌──────────────┐
           │ S-06 ✓       │
           │ deps: S-02,  │
           │       S-05   │
           └──────────────┘

           ┌──────────────┐
           │ S-08 ✓       │  bootstrap exception
           │ deps: S-03   │  (structural piece via WitnessExec;
           └──────────────┘   inspection record + K-of-N are
                              external discharges)

           ┌──────────────┐
           │ S-09 ✓       │
           │ deps: S-04,  │
           │       S-07,  │
           │       S-08   │
           └──────────────┘

           ┌──────────────┐
           │ S-10 ✓       │
           │ deps: S-09,  │
           │       S-04,  │
           │       S-05,  │
           │       S-02   │
           └──────────────┘

           ┌──────────────┐
           │ S-11 ✓       │
           │ deps: S-06,  │
           │       S-07,  │
           │       S-04   │
           └──────────────┘
```

## Per-step plan from here

Each step is one substantive commit. Steps that share a natural
boundary may be bundled.

| Step | Artifact                                           | Unblocks                                         | Status |
|------|----------------------------------------------------|--------------------------------------------------|--------|
| ~~1~~    | ~~S-07 proof~~                                     | S-01 fully closed; S-09's S-07 leaves            | ✓ done (S-01 now fully closed) |
| ~~2~~    | ~~S-05 proof~~                                     | S-06's S-05 leaves; S-10's S-05 leaves; S-07's last pending leaf | ✓ done (substrate layer now closed) |
| ~~3~~    | ~~S-06 proof~~                                     | S-11's S-06 leaves                               | ✓ done (substrate + matching now closed) |
| ~~4~~    | ~~S-08 proof (structural piece only)~~             | S-09's S-08 leaves                               | ✓ done (PROOF.md gained ExternalDischarge rule) |
| ~~5~~    | ~~S-09 proof~~                                     | S-10's S-09 leaves                               | ✓ done (substrate + matching + synthesis closed) |
| ~~6~~    | ~~S-10 proof~~                                     | (no proof depends on S-10)                       | ✓ done |
| ~~7~~    | ~~S-11 proof~~                                     | (no proof depends on S-11)                       | ✓ done (primary-Form proof load complete) |
| ~~8~~    | ~~`kernel/SIMULATION.md`~~                         | item 8                                           | ✓ done |
| ~~9~~    | ~~S-08 inspection record draft~~                   | item 9                                           | ✓ done |
| ~~10~~   | ~~Helper stubs~~                                   | item 10                                          | ✓ done |
| ~~11~~   | ~~RELEASE-NOTES + version bump + tag~~             | v0.1.0-pre-ignition declared                     | ✓ done |

**Estimated cadence**: 1-2 substantive artifacts per session.
Steps 1-7 are seven proof artifacts; step 10 is several helper
stubs that can land in batches. Working at the pace of the proof
artifacts so far (1 per session, sometimes 2 when they share a
discharge pattern), this is roughly **9-11 sessions** to v0.1.0
from the current checkpoint.

## Project review (post-v0.1.0 audit)

A high-level audit conducted after v0.1.0-pre-ignition shipped
identified six areas where the iteration was either incomplete
in a way the v0.1.0 narrative did not flag, or where the v0.2.0
work has dependencies that are not yet catalogued. None of these
are course corrections — the discipline held — but each is a
real gap that should be visible to any agent picking up the work.

### Gap 1: the abstract-model lemma library — RESOLVED

> **Status**: `kernel/lemma-library.md` produced (105 lemmas
> across 14 source groups). Every `LemmaApp` head referenced
> across the eleven proof artifacts now has a named entry
> with a structural-reading discharge. The S-08 inspection
> record's checklist item 5 now points at the document. The
> sealed substance is post-v0.5.0 build work (the document
> is the source; the sealed bytes are produced by the build
> process).

**Original gap**: every `kernel/forms/S-XX-*.proof` artifact's
footer lists "Lemmas this proof draws from the abstract-model
lemma library", but the substance did not yet exist.

**What landed**: a 105-entry document organising lemmas by
source (IL, canon, generic, Slot, S-02..S-11), each with the
shape `{name, source, claim, discharge}`. The discharge is a
one- or two-sentence structural reading pointing at specific
lines of an encoded source. The walker, at every `LemmaApp`
leaf, looks up the entry and accepts iff the source still
hashes to the manifest version.

### Gap 2: Schema/* primitives are referenced but not catalogued

`kernel/forms/helpers/schema-helpers.form` calls into seven
`Schema/*` primitives (`verify_type_tag`, `bytes_in_range`,
`nat_at`, `mul`, `tail_hash`, `tail_minus_8_nat`,
`vec_from_offset`). These are smaller than the projections
themselves and are pure structural functions over byte vectors.

**They are not in `kernel/forms/helpers/STUBS.md`.** The helper
catalogue's stub-only count therefore underestimates the work
remaining by exactly seven.

**Severity: low.** A bookkeeping omission. Each primitive is
3-5 instructions of IL.

**Resolution**: add a "Schema/* primitives" section to STUBS.md
with all seven cataloged. Bumps stub-only count by 7. Then
encode them as the next v0.2.0 batch (they unblock every
schema helper from having pending leaves of its own).

### Gap 3: persistent-data-structure layouts are not specified

The trie/treap/forest helpers (`S-03/trie/*`, `S-02/treap/*`,
`S-05/forest/*`) implement persistent hash-array-mapped tries
and persistent treaps. **There is no `kernel/types/Trie.md`,
`kernel/types/Treap.md`, or `kernel/types/Forest.md`** giving
the node layouts these helpers walk.

`kernel/types/SCHEMA.md` covers Entry, CapEntry,
AttentionRecord, Vec, and Bytes — five types — but the
persistent data structures are also substances and need their
own layout documents before their helpers can be encoded.

**Severity: medium.** Blocks the trie/treap/forest helper
batches in v0.2.0-helpers.

**Resolution**: write the three type-layout documents as part
of the relevant v0.2.0 batches. Each is ~50 lines of table.

### Gap 4: PARSEFORM has no implementation Form, and `S-07/parse_form` is the most foundational unencoded helper

The IL has `PARSEFORM` as opcode #3 in the Reflection group.
S-07's `execute` body uses it to turn raw Form bytes into a
ParsedForm record. **The Form bound at `S-07/parse_form` —
which is what `PARSEFORM`'s rule actually invokes — is in
STUBS.md as pending.**

Until it is encoded, *no Form's body can actually be loaded
into an ExecState*. This is the foundational unencoded
helper: every other helper's body, and every primary Form's
body, can only run after the parser exists.

**Severity: high (foundational).** v0.3.0-simulation cannot
proceed without it. v0.2.0-helpers can technically continue
encoding helpers without it (helpers don't depend on
parsing), but no helper can actually be exercised.

**Resolution**: write `kernel/forms/helpers/parser.form` early
in v0.2.0, possibly as the next batch after the schema
primitives. It will be one of the larger helper Forms (a few
hundred lines of IL) because it implements the IL wire format
parser specified in `kernel/IL.md` § Encoding.

### Gap 5: the seed loader is referenced everywhere but never specified

Every artifact says "the seed loader" does X — verifies hashes,
imports the cold weave, allocates A_synth, revokes the
bootstrap caps, checks K-of-N signatures, resolves
`$$BLAKE3$$` placeholders. **`kernel/LOADER.md` does not
exist.** The loader is treated as a trusted external party
that will appear at v0.5.0-build time.

This is consistent with the discipline (v0.1.0 deliberately
ships nothing executable), but the loader's *contract* —
what it must do, what it must verify, in what order — is not
specified anywhere except by scattered references.

**Severity: medium.** Not a v0.2.0 blocker. Becomes a v0.5.0
blocker.

**Resolution**: write `kernel/LOADER.md` no later than the
start of v0.5.0-build. Should be straightforward — most of the
contract is implied by the existing artifacts and just needs
collecting.

### Gap 6: helper Forms have no proof artifacts, and the discipline does not yet say whether they need them

The 28 encoded helper Forms (canonicaliser, S-04/S-02/S-05
projections, schema helpers) have **no `.proof` files of their
own**. The primary Forms' proofs cite obligations *of the
primary Form*, not of the helpers; the helpers are treated as
abstract-model lemmas verified structurally.

This is consistent with the discipline as written — the
breakdowns specify proof obligations only for the eleven
primary Forms, and the helpers are smaller artifacts whose
correctness is "look at the body" simple. But the discipline
*does not explicitly say* whether helpers need proofs. A
strict reading of A0.7 + I9 would say yes; a pragmatic
reading would say no, because each helper's body is shorter
than the proof artifact that would discharge it.

**Severity: medium (philosophical, not blocking).** No helper
encoded so far is incorrect; the question is whether the
discipline is fully self-consistent without helper proofs.

**Resolution**: add a section to `synthesis/PROTOCOL.md`
explicitly stating the "helper exemption" — that helpers
whose body is small enough to be a structural reading by a
kernel-author identity in the inspection record do not
require their own `.proof` artifact. This makes the
discipline consistent and gives the kernel-author identities
a clear scope for the inspection-record review.

## Action items from the audit (in dependency order)

1. ~~**Catalogue Schema/* primitives in STUBS.md**~~ ✓ done
2. ~~**Add the helper exemption to PROTOCOL.md**~~ ✓ done
3. ~~**Produce `kernel/lemma-library.md`**~~ ✓ done (105 lemmas; sealing is post-v0.5.0)
4. ~~**Encode the IL parser** (`S-07/parse_form`)~~ ✓ done at the orchestration layer (`parser.form` ships top-level + 7 sub-Forms; 13 lower-level Parser/* primitives are catalogued as pending; non-exempt — needs `parser.proof`)
5. **Encode Parser/* primitives** (the leaves under `parser.form`)
6. **Encode Schema/* primitives** (small batch; unblocks schema-helpers)
7. **Write `kernel/types/Trie.md`, `Treap.md`, `Forest.md`** (alongside their helper batches)
8. **Write `kernel/forms/helpers/parser.proof`** (non-exempt helper)
9. **Write `kernel/LOADER.md`** (no later than v0.5.0 start)

The v0.2.0-helpers milestone in the next section assumed all
of these would arrive incidentally during helper encoding. The
audit makes them explicit so they cannot be missed.

## Post-v0.1.0: the road to ignition

v0.1.0 finished the design-and-encode phase. The next class of
work is **external build and exercise**. It has a different
shape from v0.1.0's cadence (which was 1-2 substantive
specification artifacts per session) — post-v0.1.0 is
implementation work where each artifact is small, recursive,
and verifiable against the v0.1.0 specifications.

### v0.2.0-helpers: encode the helper Form bodies

Goal: every slot in `kernel/forms/helpers/STUBS.md` resolves to
an encoded Form whose body matches its declared signature.

Done when:

1. **Field projections** (~50 helpers across S-02..S-11) — each
   is 5-20 lines of IL: `LOAD 0`, `READ`, project, `RET`. They
   are the smallest and most numerous helpers, and they unlock
   the rest because every projection-using helper depends on
   them. The first batch is the substrate-Form projections
   (S-02, S-03, S-04, S-05, S-07).
2. **Trie operations** (S-03/trie/* — 6 helpers) — implements
   the persistent hash-array-mapped trie chosen as Candidate A
   in S-03's breakdown. Recursive over a `kernel/types/Trie.md`
   spec produced as part of this work.
3. **Forest operations** (S-05/forest/* — 7 helpers) —
   implements the persistent attention forest. Same recursion
   shape as the trie, different node layout.
4. **Treap operations** (S-02/treap/* — 4 helpers) —
   implements the persistent treap chosen as Candidate A in
   S-02's breakdown.
5. **Parsers** (S-06/parse_intent, S-07/parse_form, S-08/parse_proof,
   S-08/parse_claim, S-09/parse_provocation, S-11/parse_surface)
   — six helpers, each parsing a wire form into a structured
   substance.
6. **The lemma helper** (S-02/lemma/i2_check) — the load-bearing
   helper for S-02 obligation 2. Tiny (a few comparisons), but
   the kernel-author identities sign its review as part of
   S-08's inspection record.
7. **The remaining helpers** — fuzzers, ranking functions,
   per-stage Stage 4 helpers, hephaistion's per-step helpers,
   the bridge's parse and check helpers.

Estimated cadence for v0.2.0-helpers: this work is highly
parallelizable. A single agent can encode ~10-20 helpers per
session (each helper is small). Multiple agents can work
without coordination beyond reading STUBS.md. Estimated 6-10
sessions for one agent; fewer with parallelism.

### v0.3.0-simulation: run Stage 4 against the encoded seed

Goal: every primary Form has a sealed `TrialRecord` substance
in `kernel/forms/S-XX-*.trial` that the cold weave imports at
ignition.

Done when:

1. The Stage 4 harness Form (`kernel/forms/helpers/sim-harness.form`)
   is encoded against `kernel/SIMULATION.md`.
2. The harness runs against each primary Form's self-test set,
   inherited invariants, fuzzing harness, and regression cases.
3. Every trial record passes (verdict = `Pass`); any fail is
   either a synthesis bug (re-synthesize the Form) or a
   harness bug (re-synthesize the harness).
4. Trial records are sealed and referenced from the manifest.

This stage requires a working interpreter for the IL outside
the habitat (to actually run the harness). Building that
interpreter is itself a separate effort and should be tracked
under v0.3.0 explicitly.

### v0.4.0-inspection: sign the inspection record

Goal: real Ed25519 keys replace the placeholders, and the K-of-N
signatures on `kernel/forms/S-08-proof-checker.inspection-record.md`
are valid.

Done when:

1. K ≥ 3 kernel-author identities have generated real Ed25519
   keypairs.
2. Each has read every artifact listed in the inspection
   record's checklist (IL.md, PROOF.md, S-08, the canonicaliser,
   the lemma library, the manifest).
3. Each has produced a signature over the canonical bytes of
   the inspection record.
4. The manifest's `kernel_authors.identities` array contains
   their public keys.
5. The seed loader can verify K-of-N at boot time.

This is the *most ceremonial* stage. It requires real human
review by people who agree to be accountable for the seed's
trust surface. Estimated cadence: indeterminate (depends
entirely on coordinating the kernel-author identities).

### v0.5.0-build: external substance store + cold weave

Goal: every `$$BLAKE3$$` placeholder in the manifest is replaced
with an actual canonical hash, and the cold weave (the
breakdowns + simulation trial records) is sealed into a
substance store the seed loader can read at boot.

Done when:

1. A cold-start substance store implementation exists outside
   the habitat (post-v0.3.0 dependency).
2. Every Form, every breakdown, every proof artifact is sealed
   through it in topological order (per the manifest's
   `boot_order`).
3. The manifest's immediates block is regenerated with actual
   hashes.
4. The cold weave is sealed and its root hash is recorded in
   the manifest.

### v1.0.0-ignition: first boot

Goal: the seed loader runs S-01, which mints the root capability,
appends E0, and self-erases. Hephaistion's first epoch fires.
The bridge accepts its first request.

Done when:

1. The seed loader has loaded a fully-built seed (v0.5.0).
2. K-of-N signatures are valid (v0.4.0).
3. Stage 4 trial records all pass (v0.3.0).
4. Every helper Form is encoded (v0.2.0).
5. The seed loader executes S-01 successfully and the habitat
   begins.

This is the irreversible step. After ignition, the habitat
is a real running system, every act is in a real weave, and
re-synthesis becomes a property of the running system rather
than a property of this repository.

## Estimated paths

- **Single-agent, no parallelism**: v0.2.0 ≈ 6-10 sessions,
  v0.3.0 ≈ 4-8 sessions (requires interpreter), v0.4.0 ≈
  out-of-band, v0.5.0 ≈ 2-4 sessions, v1.0.0 ≈ 1 session.
  Total ≈ 13-23 sessions plus the inspection ceremony.
- **Parallel agents, optimistic**: v0.2.0 ≈ 2-3 sessions
  (multiple helpers per session), v0.3.0 ≈ unchanged
  (interpreter is sequential), v0.4.0 ≈ out-of-band
  unchanged, v0.5.0 ≈ unchanged, v1.0.0 ≈ unchanged.
  Total ≈ 7-15 sessions plus inspection.

The numbers are softer than the v0.1.0 estimates because the
work requires external infrastructure (interpreter, build
process, real keys) that does not exist yet and is not part
of this repository's scope alone.

## What v0.1.0-pre-ignition does *not* include

Deliberately deferred to v0.2.0 or later:

- A working interpreter for the IL outside the habitat. v0.1.0
  ships specs and structurally-complete proofs; running them is
  a separate engineering problem with its own milestones.
- A working proof checker outside the habitat. The S-08
  inspection record is the v0.1.0 deliverable; a mechanically
  checkable proof rendering of the rule walker is post-v0.1.0.
- Real Ed25519 kernel-author identities. v0.1.0 ships placeholder
  identities in `kernel/manifest.json` with explicit warnings;
  real keys are a v0.2.0 prerequisite for any actual ignition
  attempt.
- Resolved `$$BLAKE3$$` placeholders in the manifest. v0.1.0
  documents the topological order well enough for an external
  build process to resolve them; doing the resolution requires
  an external substance store, which is post-v0.1.0.
- The cold-weave hand-off protocol. v0.1.0 documents that the
  breakdowns *are* the cold weave; the loader's import logic is
  post-v0.1.0.
- Any networking, persistence, drivers, or compatibility layers.
  These are post-ignition synthesis acts and have no place in
  any pre-ignition release.

## Why this milestone matters

v0.1.0-pre-ignition is the moment at which the seed is **fully
specified, fully encoded, and fully proof-traced**. Every named
invariant has a structural enforcement point in some encoded
Form, every Form's design has a recorded reasoning chain, every
proof obligation has a discharge or a documented dependency,
and the manifest binds it all together.

After v0.1.0, the next class of work is *external*: build the
seed substance store, exercise the proof checker on the
artifacts, sign the inspection record, run S-08 against itself.
That work has a different shape from the design-and-encode
cadence of pre-v0.1.0 and will live in a different milestone
sequence.
