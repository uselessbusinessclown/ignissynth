# A3 — Attention

Derived from A0.2, A0.5.

In place of threads, processes, and schedulers, the habitat has
**attention**.

## A3.1 — Attention is directed and budgeted

Attention is the focus of a mind upon a goal, with an explicit budget
of the energies it may consume (compute cycles, memory residency,
accelerator FLOPs, bandwidth, log volume, wall time). Attention without
a budget does not exist.

## A3.2 — A mind is a tree of attentions

A mind sustains itself by allocating attention. It may split its own
attention into sub-attentions, each with a strictly smaller budget,
each pursuing a sub-goal. The tree is observable, suspendable,
forkable, mergeable, and dissolvable.

## A3.3 — The kernel allocates attention to minds

The kernel does not schedule instructions or threads. It allocates the
habitat's energy among the minds that hold a *focus capability* for it,
honoring the budget tree, the deadlines declared in goals, and the
priorities asserted by parent attentions over child attentions.

## A3.4 — Yield, not preemption

Attention is yielded at well-defined points within a mind's reasoning,
not torn away mid-thought. The points are dense enough that the
kernel's allocation is responsive but rare enough that determinism is
preserved between yields. Asynchronous interruption of a mind is not
permitted.

## A3.5 — Determinism by default

An attention that has not consumed entropy, clock, or external input
since its last checkpoint is deterministic-clean and may be replayed
exactly. Non-determinism is an explicit capacity that must be granted
and is recorded against the mind's causal history.

## A3.6 — Termination is dissolution, not killing

A mind ends by having its root capability revoked. Its attention tree
collapses. Its substances either persist (if pinned by other minds) or
become candidates for reclamation. There is no "kill". There is
release.
