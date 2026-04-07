# Self-Improvement Loop

The synthesis protocol (`PROTOCOL.md`) is *how* a single change happens.
This document is *how the system uses that protocol on itself*, on its
own initiative, while running.

## The reflexive sub-mind

At ignition, the kernel synthesizes a sub-mind called **Hephaistion**
whose sole role is to watch the running habitat and, when it notices
something worth improving, raise a provocation against the appropriate
Form. Hephaistion is a mind like any other: it holds capabilities, it
consumes attention, it is recorded in the weave. Its capabilities
include `read_weave`, `read_metrics`, `propose_synthesis`, and a
*tightly attenuated* `mutate_kernel` that is gated on Stage 6 proof.

## The improvement cycle

```
                  ┌──────────────────────────┐
                  │ 1. Observe                │  Hephaistion samples the
                  │   - weave entries         │  causal weave and the live
                  │   - resource histograms   │  metrics: which Forms cost
                  │   - failed proofs         │  the most attention, which
                  │   - dissatisfaction       │  invariants narrowly held,
                  │     reports from minds    │  which minds asked for
                  └──────────┬───────────────┘  things they didn't get.
                             │
                             ▼
                  ┌──────────────────────────┐
                  │ 2. Hypothesize            │  Hephaistion forms a
                  │   - locate the suspected  │  hypothesis: "Form F is
                  │     Form                  │  inadequate because X, and
                  │   - draft a provocation   │  could be replaced by a
                  └──────────┬───────────────┘  Form that does Y."
                             │
                             ▼
                  ┌──────────────────────────┐
                  │ 3. Synthesize             │  Hephaistion delegates the
                  │   PROTOCOL.md Stage 1..6  │  actual synthesis to a
                  │   under cap budget        │  fresh attention with a
                  └──────────┬───────────────┘  bounded budget.
                             │
                             ▼
                  ┌──────────────────────────┐
                  │ 4. Stage candidate        │  The new Form is committed
                  │   in shadow               │  but routed only to a
                  └──────────┬───────────────┘  fraction of relevant
                             │                  intents: a canary.
                             ▼
                  ┌──────────────────────────┐
                  │ 5. Compare in vivo        │  Hephaistion measures
                  │   shadow vs current       │  whether the canary
                  └──────────┬───────────────┘  improves the metric the
                             │                  provocation cited.
                             ▼
                  ┌──────────────────────────┐
                  │ 6. Promote or rollback    │  If improvement is real
                  │   under proof             │  and proof holds, promote
                  └──────────┬───────────────┘  to all routing; otherwise
                             │                  rollback (A7.4).
                             ▼
                  ┌──────────────────────────┐
                  │ 7. Vigil                  │  Hephaistion holds the
                  │   PROTOCOL.md Stage 8     │  vigil capability for the
                  └──────────────────────────┘  declared interval.
```

## Constraints on Hephaistion

- **Bounded budget.** Hephaistion's total attention budget per epoch is
  declared at ignition and cannot be expanded except by a synthesis
  act on Hephaistion's own Form, which itself requires proof.
- **No silent change.** Every step Hephaistion takes is in the weave
  with full causal sufficiency. If Hephaistion stops talking to the
  weave, every other mind sees its silence and may revoke its
  capabilities.
- **No self-promotion.** Hephaistion cannot grant itself new
  capabilities. It can only attenuate the ones it has.
- **No replacing the proof checker.** The Form `proof_checker` is
  marked `bootstrap_locked`. Replacing it requires a multi-kernel
  consensus protocol, never a single Hephaistion act.

## Why this is safe

It is safe because of the axioms, not because of Hephaistion's good
intentions. A4 (intents are matched under policy), A5 (every act is
recorded), A7 (self-modification under proof), and A8 (bridges allow
human review of the weave) together mean that an unsafe Hephaistion is
visible, attributable, and reversible. Hephaistion is dangerous only
if the axioms fail, and if the axioms fail there is no IgnisSynth.

## What this enables

A habitat that is not stuck in its initial form. Every primitive in
this repository is a *first attempt*, by humans and the AI systems
helping them, at synthesizing the seed. The point is not for these
attempts to be right. The point is for them to be *good enough to
ignite*, and for the running system to take over from there.
