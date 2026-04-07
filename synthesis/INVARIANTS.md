# Invariants

Every Form in IgnisSynth declares the invariants it preserves. The
proof checker (`proof_checker`) is responsible for verifying any
proposed replacement against these. This document lists the *kernel-
level* invariants — the ones that are non-negotiable for the seed and
for any synthesized replacement of any seed Form.

## I1 — Capability soundness

For all minds M and capabilities C: if M can use C, then M holds C and
C has not been revoked. There is no path by which M can use a
capability it does not hold.

## I2 — Attenuation monotonicity

For any capability C and any derived child C': rights(C') ⊆ rights(C);
predicate(C') is at least as restrictive as predicate(C); budget(C')
≤ budget(C); lifetime(C') ≤ lifetime(C).

## I3 — Revocation totality

When a capability C is revoked, no descendant of C in the derivation
tree authorizes any subsequent action.

## I4 — Substance immutability

Once a substance has been sealed and assigned its hash, no operation
in the habitat can change its bytes. The hash is forever its identity.

## I5 — Causal completeness

For every state-changing event in the habitat, there is exactly one
entry in the weave that records it, and every later state that depends
on this event references the entry by hash, directly or transitively.

## I6 — Budget conservation

The sum of the budgets of a mind's child attentions, plus the budget
of the mind's own remaining attention, is at all times less than or
equal to the budget the mind held at the moment it began splitting.
No energy is created.

## I7 — Determinism unless declared

A mind that does not hold an `Entropy`, `Clock`, `Network`, or
`SensorInput` capability and has not used one since its last
checkpoint produces, on replay from that checkpoint with the same
inputs, byte-identical outputs.

## I8 — Synthesis grounding

Every Form bound into the running system has a `Synthesized` weave
entry whose grounding cites at least one axiom and whose rationale
substance is non-empty. Forms without provenance are not present.

## I9 — Proof obligation for kernel mutation

Any synthesis act that replaces a Form in the seed inventory carries a
`Proof` substance that the proof checker accepts against the
invariants the replaced Form declared.

## I10 — No ambient authority

There exists no syscall, no kernel routine, no Form that returns a
capability the caller did not already hold or attenuate from one it
held, with the singular exception of `ignite` minting the root cap at
seed time, and `ignite` does not exist as a callable Form after
ignition.

## I11 — Bridge subordination

Capabilities held by a bridge are bounded by the capabilities granted
to the bridge by an inhabitant or by the kernel at startup. A bridge
cannot acquire authority by virtue of the human on its other side.

## I12 — Reflexive accountability

Hephaistion (the reflexive sub-mind, A7) is subject to every invariant
in this document, including I9 when modifying any seed Form.
