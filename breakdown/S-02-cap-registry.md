# S-02: `cap_registry`

A worked example of a seed task. Use this as a model for the others.

## Provocation

```
Provocation {
    author:     "ignis-seed-synthesizer-0"
    statement:  "The habitat must mediate authority. Without a registry of
                 capabilities and a revocation tree, axiom A2 cannot hold.
                 No mind can be safely given anything until this Form
                 exists."
    observed:   ["seed"]
    constraint: [
        "every operation O(log n) in live caps",
        "revocation cascades to descendants in a single act",
        "no ambient authority anywhere",
        "fits within the seed line budget for cap_registry (~1500 lines of Form IL)"
    ]
}
```

## Grounding

- Axioms cited: A2.1, A2.2, A2.3, A2.4, A2.5, A2.6, A0.3, A0.4
- Forms touched: none (this is seed-original)
- Invariants inherited: I1 (capability soundness), I2 (attenuation
  monotonicity), I3 (revocation totality), I10 (no ambient authority)

## Candidate(s)

### Candidate A — Persistent treap

- Sketch: a persistent treap keyed by `CapId` (BLAKE3 hash of the
  capability preimage). Each node carries `{type, rights, parent,
  predicate, holder, generation}`. Revocation bumps a generation
  counter on a subtree root, making lookups O(log n) and revocation
  O(log n). Persistence makes scheduler checkpoints O(1).
- Self-test set:
  - mint → lookup returns the same metadata
  - attenuate → child rights ⊆ parent rights
  - revoke parent → descendants fail lookup with EREVOKED
  - random op fuzzing for 10⁶ ops, no invariant violation
- Declared invariants: I1, I2, I3

### Candidate B — Sparse hash table with revocation epoch list

- Sketch: a flat hash table from `CapId` to entry, plus a per-cap
  revocation epoch. Cheaper constant factors, but revocation cascades
  cost O(descendants) without help, requiring an additional auxiliary
  index that adds memory overhead. Cheaper to read, more expensive to
  revoke.
- Self-test set: same as A.
- Declared invariants: I1, I2, I3.

## Rationale

A2.3 requires that revocation extend to all descendants. A2.6 requires
that lookups be efficient enough that authority checks are not a
significant fraction of attention budget. A0.5 forbids waste.

Candidate A pays a small constant factor for treap operations and
makes revocation a single generation bump on a subtree root, which
satisfies A2.3 in time bounded by the depth of the derivation tree.
Candidate B pays a smaller constant factor for individual lookups but
requires either O(descendants) cascade cost or auxiliary memory.

The seed faces an unknown workload. Hephaistion (S-10) is expected to
re-synthesize this Form once the actual workload is observable. The
question is therefore not "which is best in steady state" but "which
is least likely to embarrass the axioms in the first hour". Candidate
A's worst-case behavior is bounded by tree depth, which is a property
the synthesizer can reason about; Candidate B's worst case is bounded
by descendant count, which the seed has no way to predict.

We select Candidate A.

## Simulation record

(stub — produced by the Stage 4 harness once the candidate is encoded
in Form IL)

## Selection

Candidate A, by the criterion "least bounded worst case under unknown
workload" declared in the constraint list.

## Proof

Required (this Form is in the seed inventory, I9). The proof must
show:

1. For all mint operations: the returned id is unique and the entry
   is reachable.
2. For all attenuate operations: the child entry has rights, predicate,
   and budget that are pointwise no greater than the parent's.
3. For all revoke operations: every descendant of the revoked node
   fails lookup at any time after the revocation.
4. For all lookup operations: the returned entry is the most recent
   un-revoked one for the given id.

The proof is to be discharged in the proof checker's input language,
mechanically checked, and committed alongside the Form.

## Vigil declaration

Holder: `hephaistion-seed`. Duration: until 10⁹ capability operations
have completed without invariant violation. Anomaly thresholds:

- any I1 violation → immediate rollback
- any cap operation taking >1024 attention units → re-synthesis
  provocation
- any revocation cascade taking >tree depth + 1 steps → re-synthesis
  provocation
