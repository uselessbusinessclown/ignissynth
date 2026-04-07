# The Seed

The seed is the smallest set of Forms that, once ignited, can sustain
the synthesis protocol against itself. After ignition, every other
Form in the habitat is supposed to arrive through synthesis. The seed
is the only thing in the habitat that is not the product of synthesis,
because the protocol that produces synthesis is in the seed.

The seed is therefore the most carefully justified part of the system.
It is also kept as small as possible: anything that can be deferred to
post-ignition synthesis is deferred.

## Seed inventory

| Form               | Role                                          | Size target |
|--------------------|-----------------------------------------------|-------------|
| `ignite`           | The ignition routine. Verifies the seed       |       300 |
|                    | manifest, mints the root cap, hands off.      |             |
| `cap_registry`     | The kernel's table of capabilities and the    |      1500 |
|                    | revocation tree.                              |             |
| `substance_store`  | Content-addressed sealed substance store      |      1200 |
|                    | with pinning. RAM-backed at seed time.        |             |
| `weave_log`        | Append-only Merkle log of every act.          |       800 |
|                    | The system's own memory of itself.            |             |
| `attention_alloc`  | The kernel's allocator of attention to        |      1500 |
|                    | minds, honoring budget trees and yields.      |             |
| `intent_match`     | Matches submitted intents to fulfillers       |       900 |
|                    | under a policy capability.                    |             |
| `form_runtime`     | Materializes a Form into an attention's       |      1200 |
|                    | working set; the only "loader" in the system. |             |
| `proof_checker`    | Verifies a `Proof` substance against a Form   |      2000 |
|                    | and a set of invariants. Bootstrap-locked.    |             |
| `synth_kernel`     | The minimal kernel side of the synthesis      |      1500 |
|                    | protocol: provocation, grounding, simulation, |             |
|                    | selection, proof, commit, vigil.              |             |
| `hephaistion_seed` | The reflexive sub-mind seed (A7, SELF-IMPROVE)|       800 |
| `bridge_proto`     | The minimal typed bridge for human contact.   |       500 |

Total seed budget: ≤ **12,200 lines** of synthesized Form, expressed in
the habitat's intermediate language. This is *not* a target for human
hand-written code; it is a target for the *output of synthesis* the
first time the seed is produced.

## How the seed is produced

The seed cannot be synthesized by the running habitat (because there
is no running habitat yet). It is produced by an *external synthesis
event*: an AI reasoning system, working from `axioms/A0..A8` and
`synthesis/PROTOCOL.md`, generates each seed Form, attaches its
rationale, runs simulations against the protocol's Stage 4 harness,
and records every step in a *cold weave* — a weave entry stream that
the seed will read at first boot and adopt as its own ancestry.

The cold weave is the only thing that allows the system to be
genuinely able to say, after ignition, "here is why I am the way I
am". Without it the seed would be a black box and the axioms would be
violated for the seed itself.

## Ignition sequence

1. **Verify.** `ignite` reads the seed manifest, checks every Form
   hash, checks the cold weave's signatures.
2. **Mint.** `ignite` mints the root capability and hands it to a
   minimal bootstrapping mind.
3. **Lay the weave.** The bootstrapping mind imports the cold weave
   into the live `weave_log` as the system's prehistory.
4. **Ignite Hephaistion.** The bootstrapping mind summons
   `hephaistion_seed` as its first child attention, attenuated to its
   declared budget.
5. **Drop root.** The bootstrapping mind revokes its own root
   capability and dissolves. From this moment forward, no one in the
   habitat holds ambient authority.
6. **Live.** Hephaistion begins its observation cycle. The habitat is
   alive.

## What the seed deliberately does not contain

- A network stack. Any network reachability is a post-ignition
  synthesis.
- A storage backend beyond RAM. Persistence to durable substrate is a
  post-ignition synthesis under capability gating.
- A driver framework. Drivers are post-ignition synthesis.
- A scheduler tuned for any specific workload. The seed
  `attention_alloc` is the simplest correct allocator; Hephaistion is
  expected to replace it within hours of ignition with something
  better suited to the actual inhabitants.
- A compatibility layer for anything. Ever.
