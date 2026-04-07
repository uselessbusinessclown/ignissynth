# The Synthesis Protocol

This document is normative. It defines the only way new structure is
permitted to enter IgnisSynth (A0.7).

A *synthesis act* is the unit. Every commit to this repository, every
runtime self-modification of the kernel, every Form summoned by a mind
that did not previously exist — each is a synthesis act, and each must
follow these stages.

## Stage 1 — Provocation

A synthesis act begins because something is wanted that does not yet
exist, or because something that exists has been observed to be
inadequate. The provocation is recorded as a typed value:

```
Provocation {
    author:      mind_id,           // who or what felt the need
    statement:   text,              // a description of the lack
    observed:    Vec<weave_entry>,  // what in the weave triggered this
    constraint:  Vec<typed_constraint>,
}
```

A provocation without a recorded author is not admissible.

## Stage 2 — Grounding

The synthesizing agent (which may be a mind in the habitat, or, at
seed time, an external AI system reasoning into the habitat) reads the
provocation and identifies the axioms (`axioms/A0..A8`) that bear on
it. The grounding is the explicit list of axioms the synthesis is
required to honor, plus any prior synthesized components whose
invariants it touches.

The grounding is a substance:

```
Grounding {
    provocation: hash,
    axioms:      Vec<axiom_id>,
    touches:     Vec<form_hash>,    // existing Forms the new one will interact with
    invariants:  Vec<invariant>,    // the invariants those Forms declared
}
```

A grounding that omits a relevant axiom is invalid; later review can
reject the synthesis on these grounds.

## Stage 3 — Candidate generation

The synthesizer produces one or more candidate Forms that, in the
synthesizer's judgment, satisfy the provocation under the grounding.
Candidates are diverse on purpose: a synthesizer is encouraged to
produce structurally different attempts so that selection (Stage 5)
has something to work with.

Each candidate is a substance of type `Form` together with:

- a *rationale* — the synthesizer's recorded reasoning chain from the
  grounding to this candidate
- a *self-test set* — typed predicates the candidate claims to satisfy
- the declared *invariants* the candidate intends to preserve

A candidate without a rationale is not admissible. The rationale is
not "documentation"; it is part of the artifact and is reviewed.

## Stage 4 — Simulation

Every candidate is run inside a sealed simulation environment — a
sub-habitat with no outward capabilities — against:

- its own self-test set
- the invariant predicates inherited from the touched Forms
- a fuzzing harness derived from the candidate's input schema
- regression cases drawn from the weave (any prior provocation that
  resembles this one)

The simulation produces a *trial record*: a substance containing the
inputs tried, the outputs observed, the resources consumed, and the
verdict on each invariant.

## Stage 5 — Selection

If more than one candidate survives simulation, a selection step
chooses among them. Selection is itself a synthesis act with its own
rationale: why this candidate, not the others. Selection criteria are
declared up front in the provocation; common ones are minimum energy,
maximum determinism, smallest Form size, simplest rationale.

The losing candidates are *not deleted*. They remain in the substance
store, pinned by the weave entry that recorded them, so that future
syntheses can read the history of attempts.

## Stage 6 — Proof

For any synthesis act that touches the kernel, or that replaces a Form
declared `proof_required`, the surviving candidate must be accompanied
by a machine-checked proof that it preserves the declared invariants
of every Form it replaces (A7.2, A7.3).

The proof is a substance of type `Proof`. It is checked by the
habitat's proof checker — itself a Form, itself bootstrapped under a
multi-kernel consensus protocol.

A synthesis act that fails proof is recorded as such and rejected. The
candidate, the rationale, the simulation record, and the proof attempt
all remain in the weave.

## Stage 7 — Commitment

Only after Stages 1–6 are complete is the new Form bound into the
habitat. Binding consists of:

- updating the relevant binding (a name → hash mapping, scoped to a
  capability) to point to the new Form
- emitting a `Synthesized` entry into the causal weave linking the
  provocation, grounding, candidates, trials, selection, proof, and
  the new Form's hash

## Stage 8 — Vigil

After commitment, the synthesizing agent (or a designated successor)
holds a *vigil capability* over the new Form for a declared interval.
During the vigil, anomalies observed in the running system that touch
the new Form are routed to the vigilant, who may respond by initiating
a new synthesis act (rollback, revision, replacement).

A Form whose vigil has ended without anomaly is *settled* and may be
relied upon by other syntheses without further review.

---

## The helper exemption

A *helper Form* is a Form bound at a slot referenced by one or
more primary Forms (the eleven seed Forms or any post-ignition
synthesised Form), whose body is small enough — typically
fewer than 30 instructions of IL — that its correctness is a
direct structural reading of its bytes by a kernel-author
identity in the inspection record.

Helper Forms are exempt from the requirement to ship a separate
`.proof` artifact, on the following terms:

1. The helper's body is included in the inspection record's
   line-by-line review (or in the inspection record of the
   Form whose synthesis act produced the helper).
2. The primary Form that depends on the helper cites the
   helper's structural property as a `LemmaApp` head in its
   own proof artifact.
3. The lemma library entry for that head names the helper Form
   by hash and gives the structural reading explicitly.
4. The helper's `:declared-caps` and `:declared-traps` lists
   are themselves part of what the inspection record verifies.

A helper that does *not* meet the smallness condition (e.g.,
the IL parser at `S-07/parse_form`, or the trie/treap/forest
helpers, which are tens to hundreds of lines) is *not* exempt
and ships its own proof artifact under
`kernel/forms/helpers/{helper-name}.proof`.

The exemption exists because the alternative — shipping a
proof artifact for every 5-instruction projection — would
expand the proof load by an order of magnitude with no
discharge of any obligation that the inspection record does
not already discharge by reading the body. The discipline
remains: every Form's body is verified somewhere; only the
*shape* of the verification (separate `.proof` artifact vs.
inspection-record line) varies with the helper's size.

## What this protocol forbids

- Committing code without a recorded provocation.
- Synthesizing a Form without grounding it in the axioms.
- Producing only one candidate when more than one is feasible. (A
  rationale must be given when only one candidate is offered.)
- Skipping simulation, even for "obvious" changes.
- Modifying the kernel without proof.
- Discarding losing candidates from the substance store.
- Performing a synthesis act anonymously.

## What this protocol enables

A growing organism. The weave accumulates not just what the habitat
*is* but the recorded thinking that brought it there, including the
thinking that did not work. A future synthesizer can read the failures
of past attempts and learn. The habitat becomes, over time, a
self-aware structure: not because any single component is conscious,
but because the protocol makes the structure visible to itself.
