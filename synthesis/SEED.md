# The Seed

The seed is the smallest set of Forms that, once ignited, can sustain
the synthesis protocol against itself. After ignition, every other
Form in the habitat is supposed to arrive through synthesis. The seed
is the only thing in the habitat that is not the product of synthesis,
because the protocol that produces synthesis is *in* the seed.

The seed is therefore the most carefully justified part of the system.
It is also kept as small as possible: anything that can be deferred to
post-ignition synthesis is deferred.

## Seed inventory

| Form               | Role                                          | Budget | Breakdown                              | Encoding                            |
|--------------------|-----------------------------------------------|--------|----------------------------------------|-------------------------------------|
| `ignite`           | The ignition routine. Mints the root cap and  |    300 | [S-01](../breakdown/S-01-ignite.md)             | [S-01](../kernel/forms/S-01-ignite.form)             |
|                    | self-erases the slot it lives in.             |        |                                        |                                     |
| `cap_registry`     | The kernel's persistent treap of capabilities |   1500 | [S-02](../breakdown/S-02-cap-registry.md)       | [S-02](../kernel/forms/S-02-cap-registry.form)       |
|                    | and the revocation tree.                      |        |                                        |                                     |
| `substance_store`  | Persistent hash-trie of sealed cells with     |   1800 | [S-03](../breakdown/S-03-substance-store.md)    | [S-03](../kernel/forms/S-03-substance-store.form)    |
|                    | pinning. The digest is identity of matter.    |        |                                        |                                     |
| `weave_log`        | Strict Merkle chain with per-substance        |   1700 | [S-04](../breakdown/S-04-weave-log.md)          | [S-04](../kernel/forms/S-04-weave-log.form)          |
|                    | back-index. The system's memory of itself.    |        |                                        |                                     |
| `attention_alloc`  | Discrete recursive-descent allocator over the |   2200 | [S-05](../breakdown/S-05-attention-alloc.md)    | [S-05](../kernel/forms/S-05-attention-alloc.form)    |
|                    | attention forest. No clock, no entropy.       |        |                                        |                                     |
| `intent_match`     | Predicate-driven match of intent to fulfiller |   1500 | [S-06](../breakdown/S-06-intent-match.md)       | [S-06](../kernel/forms/S-06-intent-match.form)       |
|                    | under a policy capability. No global index.   |        |                                        |                                     |
| `form_runtime`     | Pure interpreter over the IL. Continuations   |   2400 | [S-07](../breakdown/S-07-form-runtime.md)       | [S-07](../kernel/forms/S-07-form-runtime.form)       |
|                    | are sealed substances.                        |        |                                        |                                     |
| `proof_checker`    | Fixed natural-deduction kernel. The bootstrap |   1600 | [S-08](../breakdown/S-08-proof-checker.md)      | [S-08](../kernel/forms/S-08-proof-checker.form)      |
|                    | exception. Bootstrap-locked.                  |        |                                        |                                     |
| `synth_kernel`     | Sequential state machine over the eight       |   2000 | [S-09](../breakdown/S-09-synth-kernel.md)       | [S-09](../kernel/forms/S-09-synth-kernel.form)       |
|                    | protocol stages. One sub-attention per stage. |        |                                        |                                     |
| `hephaistion_seed` | Long-running reflexive sub-mind with          |   1700 | [S-10](../breakdown/S-10-hephaistion-seed.md)   | [S-10](../kernel/forms/S-10-hephaistion-seed.form)   |
|                    | heartbeat-as-presence and per-epoch budget.   |        |                                        |                                     |
| `bridge_proto`     | Single bridge mind, parse-only translator.    |   1400 | [S-11](../breakdown/S-11-bridge-proto.md)       | [S-11](../kernel/forms/S-11-bridge-proto.form)       |
|                    | Static cap_view, no meta-authority.           |        |                                        |                                     |

Total seed budget: ≤ **18,100 lines** of Form IL. The original target
in this document was 12,200 lines; the increase came from making
several invariants *structural* in the encodings rather than
discharging them by-discipline (the per-stage sub-attention shape in
S-09, the heartbeat-first reservation in S-10, the per-Form helper
indirection in S-03 and S-05). The discipline pays for itself in proof
load: every line of structural enforcement is a line that does not
need to appear in a proof artifact.

This is *not* a target for human hand-written code; it is a target for
the *output of synthesis* the first time the seed is produced.

## The ignition substrate (A9)

Before the seed can be ignited, a stage-0 interpreter must exist
outside the habitat. This is named as axiom A9 and specified
operationally in `kernel/IGNITION-BOOTSTRAP.md`. The stage-0
interpreter (`ignis0`) is ordinary software — Rust, OCaml, C,
or similar — that implements the 34-opcode IL and runs the
seed loader's ten-step ignition sequence.

The seed does *not* attempt to self-host at the base case.
A9 acknowledges that every runtime needs something to run it,
that the only honest response is to name that something
explicitly, and that correctness of the substrate can be
verified at ignition time via a concrete fixed-point check
(A9.3): the canonical Form `F` must produce the same result
when interpreted directly by `ignis0`, when interpreted by
S-07 under `ignis0`, and when interpreted by S-07 interpreting
S-07 under `ignis0`. Agreement is a necessary condition;
disagreement halts ignition.

`ignis0` is the substrate, not an inhabitant. It holds no
capabilities. It is not a mind. It may be replaced
post-ignition (via a checkpoint-and-restart outside the
habitat), but it may never be absent from a running habitat.
See A9.1–A9.5 and `kernel/IGNITION-BOOTSTRAP.md` for details.

## How the seed was produced

The seed in this repository was produced by an external synthesis
event: an AI reasoning system, working from `axioms/A0..A8` and
`synthesis/PROTOCOL.md`, generated each seed Form's design through a
worked synthesis act recorded in `breakdown/S-XX.md`, then encoded the
selected candidate in the IL specified in `kernel/IL.md`.

The breakdowns are the *cold weave* in document form: each one is a
complete recorded synthesis act in the shape PROTOCOL.md requires
(provocation → grounding → ≥2 candidates → rationale → simulation
record stub → selection → proof obligations → vigil declaration). At
ignition, the seed loader is required to import these breakdowns into
the live `weave_log` as the system's prehistory, so that after ignition
any inhabitant can run a why-query against any seed Form's hash and
recover the entire reasoning chain that produced it.

Without the cold weave the seed would be a black box and the axioms
would be violated for the seed itself. The breakdowns make the seed
genuinely able to say, after ignition, "here is why I am the way I am",
and Hephaistion is required to be able to read them as the same kind
of substance any other mind would.

## What the encoded forms enforce structurally

The discipline that ran consistently across all eleven encodings:
when a candidate would smuggle in (a) a hidden input that breaks I7,
(b) ambient state that breaks I10, or (c) a new authority the seed has
no provocation for, that candidate lost. Where possible, the surviving
candidate's proof obligations are discharged at the *encoding surface*
rather than in the body:

- **`:declared-caps`** is a structural cap-closure. S-08's
  `(S-03/ROOT_RO)` is the entire trust delta of the proof checker:
  read-only and nothing else. A reviewer of the inspection record
  verifies obligation #1 (purity) by reading that one line.
- **`:declared-traps ()`** is a structural totality declaration. S-08,
  S-10, and S-11 all declare empty trap sets. Every failure mode in
  these Forms is a return value or a weave entry, never a propagated
  trap.
- **Absence of opcodes** is structural enforcement of "nothing is
  ambient". The IL has no `MINT` (capability creation is the I10
  exception in S-01), no `TIME`/`NOW` (time is weave-tip order, per
  S-05), no `RAND` (entropy is a cap `INVOKE`), no `MALLOC`, no
  `SYSCALL`. Every absent opcode is a property no Form can violate
  even by accident.
- **Order of instructions** discharges some liveness obligations.
  S-10's heartbeat is appended *first* in the body, against a quantum
  the allocator reserved at ignition. So even if the rest of the cycle
  traps or loops, the heartbeat is already in the weave — obligation
  #2 (heartbeat liveness) reads off the instruction order.
- **Exports manifests** at the bottom of each `.form` file are
  type-level surfaces S-08's checker reads to discharge "no
  enumeration" obligations. The absence of a `list_*` Form anywhere in
  S-03 or S-04 or S-06 is structural because they are not in the file.

## Ignition sequence

1. **Verify.** The seed loader reads `kernel/manifest.json`, checks
   every Form source against its declared hash, verifies the cold
   weave's signatures, and verifies every `proof_obligations_status`
   entry is matched by a sealed Proof substance under
   `kernel/forms/S-XX-*.proof` (or a discharged inspection record for
   S-08 specifically).
2. **Seal.** The seed loader seals every Form substance and helper
   substance through a cold-start `substance_store` in topological
   order, producing the actual content-addressed hashes that replace
   the manifest's `$$BLAKE3$$` placeholders.
3. **Ignite.** The seed loader invokes the Form bound at the slot
   `"ignite"` (the genesis Form, S-01). Its body mints the root
   capability, appends E0 to the weave, and self-erases the slot.
   This is the only place in the entire history of the habitat where
   a capability is created without an `attenuate` from a held parent.
4. **Hand off.** The seed loader hands the root capability to the
   bootstrapping mind, which attenuates it once for each of the four
   ignition caps Hephaistion will hold (read_weave, read_metrics,
   propose_synthesis, mutate_kernel_gated) and once for the bridge's
   policy cap.
5. **Lay the weave.** The bootstrapping mind imports the breakdowns
   into the live `weave_log` as `Synthesized` entries, in topological
   order, so that the habitat starts with a complete record of its
   own design.
6. **Drop root.** The bootstrapping mind revokes its own root
   capability and dissolves. From this moment forward, no one in the
   habitat holds ambient authority. R itself remains in the cap
   registry as the genesis node, but no mind holds it.
7. **Begin.** Hephaistion's first epoch fires. The bridge's inbound
   endpoint accepts its first request. The habitat is alive.

## What the seed deliberately does not contain

- A network stack. Any network reachability is a post-ignition
  synthesis.
- A storage backend beyond RAM. Persistence to durable substrate is a
  post-ignition synthesis under capability gating.
- A driver framework. Drivers are post-ignition synthesis.
- A scheduler tuned for any specific workload. The seed
  `attention_alloc` is the simplest correct allocator; Hephaistion is
  expected to replace it within hours of ignition with something
  better suited to the actual inhabitants. The S-05 vigil explicitly
  invites this re-synthesis.
- A clock. Time, in the seed, is the order of weave entries on the
  tip. A clock Form is a post-ignition synthesis whose first effect
  is to invite re-synthesis of S-05 against Candidate B
  (token-bucket flows) — the breakdown explicitly names this
  upgrade path.
- A JIT. The seed runtime is a pure interpreter over the IL. A
  `NativeImage` substance type with substrate-stable seal is a
  post-ignition synthesis whose first effect is to invite
  re-synthesis of S-07 against Candidate B (JIT-from-IL).
- A compatibility layer for anything. Ever.
