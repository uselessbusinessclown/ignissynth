# Roadmap to v0.1.0-pre-ignition

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
| 7 | 11 proof artifacts (`kernel/forms/S-XX-*.proof`)                      | 7 of 11    |
| 8 | Stage 4 simulation harness specification (`kernel/SIMULATION.md`)     | not started|
| 9 | S-08 inspection record draft (`kernel/forms/S-08-*.inspection-record.md`) | not started|
| 10| Helper stubs for every slot referenced by an encoded Form             | 1 of N     |
| 11| `RELEASE-NOTES-v0.1.0.md` + manifest version bump + `v0.1.0` git tag  | not started|

A v0.1.0 tag is admissible when items 1-11 are checked. None of
items 1-10 require running code; item 11 requires a release notes
document and a tag.

## Proof artifact dependency graph

Each `.proof` artifact's `S0X :obligation N` leaves resolve only
when the cited proof exists. Listed in dependency order — earlier
entries unblock later ones.

```
                 ┌──────────────┐
                 │ S-03 ✓       │  the floor — no cross-Form deps
                 └──────┬───────┘
                        │
        ┌───────────────┼─────────────────────────┐
        │               │                         │
        ▼               ▼                         ▼
┌──────────────┐ ┌──────────────┐         ┌──────────────┐
│ S-04 ✓       │ │ S-02 ✓       │         │ S-07         │  pending
│ deps: S-03   │ │ deps: S-03   │         │ deps: S-03,  │
│              │ │              │         │       S-05*  │
└──────┬───────┘ └──────┬───────┘         └──────┬───────┘
       │                │                         │
       │                └──────┬──────────────────┘
       │                       │
       └───────────┬───────────┘
                   │
                   ▼
          ┌──────────────────┐
          │ S-01 ✓ (mod S-07)│  fully closed once S-07 lands
          └──────────────────┘

           ┌──────────────┐
           │ S-05         │  pending
           │ deps: S-02   │
           └──────┬───────┘
                  │
                  ▼
           ┌──────────────┐
           │ S-06         │  pending
           │ deps: S-02,  │
           │       S-05   │
           └──────────────┘

           ┌──────────────┐
           │ S-08         │  bootstrap exception
           │ deps: S-03   │  (only the structural piece is mechanizable;
           └──────────────┘   the inspection-record + K-of-N
                              discharges are out of band)

           ┌──────────────┐
           │ S-09         │  pending
           │ deps: S-04,  │
           │       S-07,  │
           │       S-08   │
           └──────────────┘

           ┌──────────────┐
           │ S-10         │  pending
           │ deps: S-09,  │
           │       S-04,  │
           │       S-05,  │
           │       S-02   │
           └──────────────┘

           ┌──────────────┐
           │ S-11         │  pending
           │ deps: S-06,  │
           │       S-07,  │
           │       S-04   │
           └──────────────┘
```

`*` = S-07's proof references S-05's `forest/cap_view` projection,
which is structurally trivial but cited as a leaf to make the
dependency explicit. This means S-07's proof becomes end-to-end
checkable only after S-05's lands; until then, S-07's proof has
one trivial pending leaf.

## Per-step plan from here

Each step is one substantive commit. Steps that share a natural
boundary may be bundled.

| Step | Artifact                                           | Unblocks                                         | Status |
|------|----------------------------------------------------|--------------------------------------------------|--------|
| ~~1~~    | ~~S-07 proof~~                                     | S-01 fully closed; S-09's S-07 leaves            | ✓ done (S-01 now fully closed) |
| ~~2~~    | ~~S-05 proof~~                                     | S-06's S-05 leaves; S-10's S-05 leaves; S-07's last pending leaf | ✓ done (substrate layer now closed) |
| ~~3~~    | ~~S-06 proof~~                                     | S-11's S-06 leaves                               | ✓ done (substrate + matching now closed) |
| **4**    | S-08 proof (structural piece only)                 | S-09's S-08 leaves                               | next   |
| **5**    | S-09 proof                                         | S-10's S-09 leaves                               | pending|
| **6**    | S-10 proof                                         | (no proof depends on S-10)                       | pending|
| **7**    | S-11 proof                                         | (no proof depends on S-11)                       | pending|
| **8**    | `kernel/SIMULATION.md` (Stage 4 harness spec)      | item 8 of v0.1.0 checklist                       | pending|
| **9**    | `kernel/forms/S-08-*.inspection-record.md` draft   | item 9 of v0.1.0 checklist                       | pending|
| **10**   | Helper stubs (parser, trie ops, lemma library, ranker, surface grammar) | item 10 of v0.1.0 checklist | pending|
| **11**   | `RELEASE-NOTES-v0.1.0.md` + version bump + tag     | v0.1.0-pre-ignition declared                     | pending|

**Estimated cadence**: 1-2 substantive artifacts per session.
Steps 1-7 are seven proof artifacts; step 10 is several helper
stubs that can land in batches. Working at the pace of the proof
artifacts so far (1 per session, sometimes 2 when they share a
discharge pattern), this is roughly **9-11 sessions** to v0.1.0
from the current checkpoint.

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
