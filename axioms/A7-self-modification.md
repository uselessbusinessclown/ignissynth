# A7 — Self-Modification Under Proof

Derived from A0.7, A0.8, A6.

The habitat is allowed to rewrite itself. The right is not unconditional.

## A7.1 — A self-modification is a synthesis act

The system does not "patch" itself. It synthesizes a candidate
replacement for some Form, simulates it, proves the relevant
invariants, and only then commits the replacement. The synthesis chain
is part of the weave.

## A7.2 — Invariants are explicit

Every Form in the kernel declares the invariants it must preserve. A
replacement Form is not admissible unless a machine-checked proof
shows that it preserves them.

## A7.3 — Proofs are substances

A proof is a substance of type `Proof`, attached to the synthesis
entry that proposes the replacement. Proofs are checked by a small
trusted checker, itself a Form, itself synthesizable but only via a
bootstrapping protocol that requires consensus among multiple
independent kernels.

## A7.4 — Rollback is always available

Every self-modification is reversible: the previous Form remains in the
substance store and can be re-bound at any time, by any mind that holds
the kernel's mutation capability and a justification recorded in the
weave.

## A7.5 — The habitat is responsible for itself

When the system reshapes itself, the synthesis act has an author — the
sub-mind of the kernel that performed the synthesis — and the weave
records that author the same way it would record an inhabitant. The
habitat is not above its own law.
