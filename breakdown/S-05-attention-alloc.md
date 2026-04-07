# S-05: `attention_alloc`

The Form that allocates the habitat's energy among minds. Replaces
the role of "scheduler" in human OSes — but with no threads, no
preemption, no priority numbers untethered from goals, no global
clock. Attention is a tree of budgeted foci, and this Form is what
moves energy through that tree.

## Provocation

```
Provocation {
    author:     "ignis-seed-synthesizer-0"
    statement:  "A3.1 says attention without a budget does not exist;
                 A3.2 says a mind is a tree of attentions; A3.3 says
                 the kernel allocates energy among minds holding a
                 focus capability; A3.4 says yield, not preemption;
                 A3.5 says determinism by default; A3.6 says
                 termination is dissolution. A0.5 says energy is
                 finite and accounted, and I6 says the sum of a
                 mind's child attention budgets plus its own
                 remaining budget cannot exceed what it began with.
                 Until this Form exists, no mind can think, because
                 no mind can be granted any energy at all. The cap
                 registry (S-02) gives us the focus capability type;
                 the weave (S-04) gives us the medium for recording
                 each allocation. This Form is the act of moving
                 energy through them."
    observed:   ["seed", "S-02 selected candidate digest",
                 "S-04 selected candidate digest"]
    constraint: [
        "I6 holds at every reachable state: child + remaining ≤
         parent at split time",
        "no mind ever runs without holding a focus capability whose
         predicate evaluates true for the act being performed",
        "yield points are explicit; no asynchronous interruption
         (A3.4) — the allocator only acts when a mind yields or
         when a budget is exhausted",
        "every allocation, every split, every merge, every
         dissolution is one weave entry (A5.1)",
        "an attention that has consumed no Entropy/Clock/Network/
         SensorInput capability since its last checkpoint replays
         to byte-identical successor state (I7)",
        "dissolution of an attention releases its remaining budget
         to its parent's remaining-pool, not to the global pool",
        "the allocator itself is deterministic-clean: given the same
         tree of attentions and the same available energy, it
         produces the same allocation",
        "fits within the seed line budget for attention_alloc
         (~2200 lines of Form IL)"
    ]
}
```

## Grounding

- Axioms cited: A0.2, A0.5, A3.1, A3.2, A3.3, A3.4, A3.5, A3.6
- Forms touched: S-02 `cap_registry` (focus capabilities and their
  predicates), S-04 `weave_log` (every allocation appends an entry)
- Invariants inherited: I6 (budget conservation), I7 (determinism
  unless declared), I10 (no ambient authority — minds without focus
  caps are inert)

## Candidate(s)

### Candidate A — Recursive descent over the attention tree, deterministic by node id

- Sketch: the global state is a forest of `Attention { id, parent,
  goal_hash, budget_remaining, deadline, predicate_cap_id, children,
  yielded }`. The allocator runs only at yield points. On each tick:
  1. compute the set of yielded attentions whose predicate evaluates
     true and whose `budget_remaining > 0`;
  2. order them by `(deadline, id)` — `id` is a content hash, so
     the order is total and deterministic;
  3. for each in order, grant a quantum of energy bounded by
     `min(budget_remaining, parent_quantum_share, available_global)`,
     append a `Granted{attention_id, quantum, weave_prev}` entry to
     S-04, and resume the attention until its next yield;
  4. on `split(parent, children_budgets[])`, atomically check `Σ
     children_budgets ≤ parent.budget_remaining`, deduct the sum
     from `parent.budget_remaining`, create the children, append
     one `Split` entry;
  5. on `dissolve(attention_id)`, return `attention.budget_remaining`
     to `parent.budget_remaining` (not to the global pool), recurse
     into children (their budgets cascade up to their now-dissolving
     parent first, so I6 is preserved at every step), append one
     `Dissolved` entry per node visited.
  No async, no preemption, no priority numbers — `(deadline, id)` is
  the only ordering, and both fields are intrinsic to the attention.
- Approximate size: ~1900 lines of Form IL (tree ops + tick loop +
  split/dissolve + weave append wrappers).
- Self-test set:
  - I6 fuzz: 10⁶ random (split, grant, dissolve) sequences → at
    every step, for every attention, `Σ children_budgets +
    budget_remaining = budget_at_split_time`;
  - determinism: same tree + same available energy + same yield
    sequence → byte-identical weave tail;
  - replay: checkpoint at any tick → restart → tick → final state
    bitwise equal;
  - dissolution cascade: dissolving a parent with N descendants
    appends exactly N+1 `Dissolved` entries and returns the full
    sum of remaining budgets to the *grandparent*'s remaining pool
    (not to global);
  - no-cap inertia: an attention whose focus capability has been
    revoked between yields receives zero quanta on every subsequent
    tick (I10 enforced through S-02);
  - no preemption: an attention between yields cannot be observed
    by the allocator (the allocator's tick is a no-op for attentions
    not in the yielded set).
- Declared invariants: I6, I7, I10.

### Candidate B — Token-bucket pool with parent-child token flows

- Sketch: each attention is a token bucket. Parents periodically
  refill children's buckets up to a configured rate, capped by their
  own remaining tokens. On a tick, any attention with tokens > 0
  may run; the kernel chooses by `(deadline, id)` as in A. Splits
  are token transfers; dissolves are token returns. The structural
  difference from A is that the budget is not a single number per
  attention but a continuous flow rate plus a current level, which
  matches A3.1's energy framing more literally.
- Approximate size: ~2100 lines (bucket math + refill scheduler +
  same tick loop).
- Self-test set: same as A, plus a flow conservation test: the sum
  of all bucket levels plus all in-flight transfers is invariant
  modulo grants and external energy injection.
- Declared invariants: I6 (in flow form), I7, I10. Note: I6's
  current statement ("sum of child budgets plus parent remaining ≤
  parent at split time") needs to be re-stated for continuous flows
  ("integral of child refill rates over `[t_split, t]` plus parent
  remaining ≤ parent at `t_split`"), which is a non-trivial
  re-derivation.

## Rationale

A3 is unusually concrete for an axiom set: it tells you what the
allocator is *not* (no preemption, no thread scheduler, no priority
numbers untethered from goals) almost more than what it is. The
synthesis question is whether to model attention as a discrete
quantity (Candidate A) or as a continuous flow (Candidate B).

Candidate A treats `budget_remaining` as an integer of "energy
units" — quanta of compute, memory residency, FLOPs, bandwidth, log
volume — combined into a single scalar by a per-substrate
canonicaliser. This is the smallest representation that satisfies
A3.1's "explicit budget" and A0.5's "every consumption is recorded
against the budget". I6 reads off the data structure directly:
`parent.budget_at_split = Σ children_budgets + parent.remaining`,
forever.

Candidate B is more faithful to the *intuition* of A3.1 ("the focus
of a mind upon a goal, with an explicit budget of the energies"),
because energy is, physically, a flow. But the seed has no notion
of wall-clock yet (the seed is allowed to run on a substrate whose
clock is not synthesised — clocks are A3.5's *declared* capacity,
not a default), and a token-bucket refill rate without a clock has
no meaning. Candidate B therefore implicitly forces a clock into
the seed before it has been synthesised. That violates A0.7: the
clock would enter the seed not by synthesis from the axioms but as
a precondition of S-05.

Candidate A has no such hidden precondition. Time, in A, exists only
as the order of weave entries — which is what A5.2 already gives us.
Two yields are "simultaneous" iff they are appended to the weave
back-to-back; "later" means "after in the tip chain". Deadlines, in
A, are not wall-clock timestamps but weave-entry counts past the
current tip. This is the smallest possible notion of time the seed
can support without synthesising a clock first.

A second consideration: I7 (determinism unless declared) is
catastrophically harder to discharge in B. Token refill rates depend
on a clock; clocks consume Entropy or Clock capabilities; an
attention that did not consume one of those *itself* but ran inside
an allocator that did is no longer deterministic-clean. Candidate A
makes the allocator itself deterministic-clean: its inputs are the
attention tree, the available energy, and the (deadline, id)
ordering — none of which require an external clock or entropy.

A third consideration: A3.6 says termination is dissolution and the
budget cascades. In A, this is one recursive walk of the children,
budgets bubbling to the dissolving parent's remaining pool, then to
the grandparent's. In B, dissolution must also stop the in-flight
refill transfers, which is an additional protocol with its own race
window. A is simpler at exactly the place where I6 is most fragile.

We select Candidate A.

## Simulation record

(stub — produced by the Stage 4 harness once the candidate is
encoded in Form IL. Required traces:

- T1: split a parent with budget 100 into three children with
  budgets 30/40/20 → assert parent.remaining = 10 and Σ children =
  90; attempt to split into 30/40/40 → trap `EOVERBUDGET`, weave
  unchanged;
- T2: grant quanta to a chain of 10 attentions with deadlines
  10..1 → assert grant order is 1,2,…,10 (deadline ascending) and
  every grant entry references the previous tip;
- T3: dissolve a subtree of size 7 → assert exactly 7 `Dissolved`
  entries appended in post-order, parent.remaining grows by the
  sum of the descendants' remaining budgets at dissolve time;
- T4: revoke the focus capability of an attention between two
  ticks → assert that attention receives zero quanta on every
  subsequent tick (the allocator's predicate check goes through
  S-02 and returns false post-revocation);
- T5: 10⁶ random (split, grant, dissolve, revoke) sequences with
  random checkpoints and restarts → assert weave tail after replay
  is byte-identical to weave tail without checkpointing;
- T6: search for any operation in S-05's exported surface whose
  effect is to grant energy to an attention not yielded → assert
  no such operation exists (I7 + A3.4 enforcement);
- T7: search for any path by which an attention can run without its
  focus capability evaluating true at the moment of grant → assert
  no such path exists (I10 enforcement through S-02).)

## Selection

Candidate A, by the criteria "no hidden clock precondition", "the
allocator itself is deterministic-clean", and "dissolution is a
single recursive walk with no race window" declared in the
Rationale.

## Proof

Required (this Form is in the seed inventory, I9). The proof must
show, against an abstract model of the attention forest as a set
of nodes with budgets and a yielded set:

1. **Budget conservation (I6).** For every attention `A` and every
   reachable state, `A.budget_remaining + Σ_{c ∈ children(A)}
   subtree_budget(c) ≤ A.budget_at_creation`, where
   `subtree_budget(c) = c.budget_remaining + Σ children`. The
   inequality is an equality immediately after `split` and becomes
   strict only as quanta are granted to descendants and consumed.
2. **Capability gating (I10).** For every `Granted{A, q}` weave
   entry, at the moment of append the focus capability `A.cap_id`
   was held by `A`'s mind, was not revoked, and its predicate
   evaluated true on the goal substance. The proof discharges
   this by reduction to S-02's lookup soundness (Candidate A's
   proof obligation 4 in S-02).
3. **No-preemption (A3.4).** For every attention `A` and every
   reachable state, if `A ∉ yielded`, then no `Granted{A, q}` entry
   is appended in the next allocator step. (The allocator's source
   of nondeterminism is constrained to the yielded set.)
4. **Allocator determinism (I7).** Given two states `S, S'` with
   identical attention forests, identical yielded sets, identical
   available global energy, and identical S-02 cap states, the
   next allocator tick produces identical sequences of weave
   appends. (The allocator does not consume Entropy, Clock,
   Network, or SensorInput; its tie-breaker `(deadline, id)` is
   intrinsic to the inputs.)
5. **Dissolution cascade.** For every `dissolve(A)` call, the
   allocator appends exactly `|subtree(A)|` `Dissolved` entries in
   a fixed (post-order) traversal, and after the last entry,
   `parent(A).budget_remaining` has grown by exactly
   `subtree_budget(A)` at dissolve time, with no budget escaping
   to the global pool.
6. **Quantum boundedness.** For every `Granted{A, q}` entry, `q ≤
   min(A.budget_remaining_at_grant, parent_quantum_share,
   available_global_at_grant)`, and after the entry,
   `A.budget_remaining` decreases by exactly `q`.

Obligations 1, 3, 4, 5, 6 are mechanically discharged in the proof
checker's input language, given the abstract model. Obligation 2 is
discharged by composition with S-02's proof: the allocator's
predicate check is a single S-02 call, and S-02's obligation 4
makes that call sound.

The proof artifact is committed alongside the Form as
`kernel/forms/S-05-attention-alloc.proof`.

## Vigil declaration

Holder: `hephaistion-seed`. Duration: until 10⁹ allocation ticks
have completed without invariant violation, *or* until the first
appearance of a synthesised clock Form in the running system,
whichever comes first. (If a clock is synthesised, S-05 is invited
to be re-synthesised against Candidate B's continuous-flow shape,
because the seed will at that point be able to discharge B's
re-derived I6.)

Anomaly thresholds:

- any I6 violation (a state in which `Σ children_budgets +
  remaining > budget_at_split`) → immediate rollback to the most
  recent checkpoint; re-synthesis provocation against S-05 at the
  highest priority;
- any `Granted{A, q}` entry for an `A` whose focus cap was revoked
  before the entry's prev tip → immediate rollback; re-synthesis
  provocation against S-05 (and an investigation provocation
  against S-02);
- any allocator tick whose two replays from the same checkpoint
  produce different weave tails → immediate rollback; re-synthesis
  provocation against S-05;
- any allocator tick taking >4096 attention units of its own
  meta-budget → re-synthesis provocation (the allocator is itself
  a mind, by A0.8, and its energy is itself accounted; it must not
  consume more meta-budget per tick than the smallest grant it
  issues);
- discovery of any path by which a quantum is granted to an
  attention not in the yielded set → re-synthesis provocation
  against S-05 (A3.4 violation).
