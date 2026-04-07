# Glossary

- **Attention** — A budgeted, directed focus of a mind on a goal. The
  unit of scheduling in IgnisSynth. (A3)
- **Axiom** — One of the irreducible statements in `axioms/`. The only
  source of normative truth in the habitat. (A0)
- **Binding** — A `name → form_hash` mapping in the kernel's slot
  table, advanced only by the `BINDSLOT` opcode under the kernel
  mutation capability. The atom of self-modification.
- **Bootstrap-locked** — A property of a Form (notably the proof
  checker, S-08) whose replacement requires the K-of-N multi-kernel
  consensus protocol rather than the standard Stage 6 proof
  discharge.
- **Breakdown** — A worked synthesis act, in document form, recording
  the provocation, grounding, candidates, rationale, simulation
  record, selection, proof obligations, and vigil declaration for one
  seed Form. The breakdowns in `breakdown/S-XX.md` are the cold weave
  the seed loader imports as the habitat's prehistory at ignition.
- **Bridge** — A mind that translates between humans and the habitat.
  (A8)
- **Canary** — A staged binding (`shadow:<name>` rather than `<name>`)
  that receives a configurable fraction of routed intents while
  Hephaistion compares its metric against the primary binding's, prior
  to a Promote-or-Rollback synthesis act.
- **Canonicaliser** — The shared Form, bound under
  `S-07/canon/normalise`, that turns any wire-form Form source into
  byte-deterministic canonical bytes. Used by both S-07 (for re-sealing)
  and S-08 (for `equiv_under_canon`).
- **Capability** — An unforgeable, attenuable, revocable token of
  authority. (A2)
- **Causal weave** — The Merkle DAG of every act in the habitat. (A5)
- **Cold weave** — The pre-ignition synthesis history of the seed
  (the breakdowns), imported as the system's prehistory at first
  boot. (`SEED.md`)
- **Continuation** — A sealed substance of type `Cont` whose contents
  are an `ExecState` from which a yielded attention can be resumed.
  Identity is hash; replaying the same continuation hash on a clean
  attention produces a byte-identical successor.
- **Digest** — The root hash of the substance store's persistent
  trie. The identity of habitat matter at any checkpoint. Composes
  with the weave tip and the active continuation hash to identify the
  entire habitat state.
- **ExecState** — The runtime tuple `{form_hash, pc, locals, stack,
  cap_view, weave_prev, inputs_hash, attention_id}` that S-07's
  interpreter steps over. Itself a substance type.
- **Form** — A substance of executable type, written in the IL.
  (A6.1, `kernel/IL.md`)
- **Form IL** — The 30-opcode intermediate language specified in
  `kernel/IL.md`, in which every encoded seed Form is written. Total
  small-step semantics, no `MINT`, no `TIME`, no `RAND`, no `MALLOC`,
  no `SYSCALL`. The smallness is load-bearing: S-08's checker reasons
  about every instruction, S-07's runtime implements them all, and
  the inspection-record discharge of S-08 covers them in one sitting.
- **Habitat** — IgnisSynth itself, considered as an environment for
  minds.
- **Heartbeat** — The first weave entry hephaistion appends each
  epoch, against a quantum the allocator reserved at ignition. Its
  presence is the positive signal of hephaistion's liveness; its
  absence is observable in O(1) by any mind holding `read_weave`.
- **Helper Form** — A non-primary Form (under `kernel/forms/helpers/`)
  bound at a slot referenced by one or more primary Forms. The
  primary Forms reach helpers by `READSLOT` + `CALL` rather than
  inlining, so each primary stays small and helpers can be
  re-synthesised independently.
- **Hephaistion** — The reflexive sub-mind that watches the habitat
  and proposes self-improvements. (`SELF-IMPROVEMENT.md`, S-10)
- **IgnisSynth** — Ignition through synthesis. The system this
  repository specifies.
- **Ignition** — The single moment at which the seed becomes a running
  habitat. The body of the genesis Form (`ignite`, S-01) runs once
  and self-erases its slot.
- **`ignis0`** — The stage-0 interpreter: an ordinary software
  artifact (in Rust, OCaml, C, or similar) that implements the
  30-opcode IL specification outside the habitat. It is the base
  case of the runtime recursion, named by axiom A9 and specified
  operationally in `kernel/IGNITION-BOOTSTRAP.md`. Not a Form, not
  a mind, not an inhabitant — the substrate the habitat runs on,
  analogous to a CPU. May be replaced post-ignition via
  checkpoint-and-restart, but never absent from a running habitat.
- **Ignition fixed-point check** — The necessary-condition test
  from A9.3 that `ignis0` must pass at ignition: direct execution
  of a canonical Form `F` must agree with execution of S-07
  interpreting `F` and with execution of S-07 interpreting S-07
  interpreting `F`. Disagreement halts ignition immediately.
- **Immediate** — A value referenced by symbolic name in a `.form`
  file (`UNIVERSAL_RIGHTS`, `BREAKDOWN_S01_HASH`,
  `PROOF_CHECKER_HASH`, etc.) that the seed loader resolves at parse
  time from the `immediates` block of `kernel/manifest.json`.
- **Inspection record** — The hand-signed line-by-line review of S-08
  by the kernel-author identities listed in `kernel/manifest.json`.
  One of the three layered discharges of S-08's bootstrap proof
  obligation; the other two are ground reflexive acceptance and
  K-of-N multi-kernel consensus on replacement.
- **Intent** — A typed declaration of a goal, inputs, constraints,
  budget, and acceptance criteria. (A4)
- **Invariant** — A property declared by a Form that any replacement
  must preserve. The twelve kernel-level invariants are I1..I12 in
  `synthesis/INVARIANTS.md`.
- **Manifest** — `kernel/manifest.json`. Binds every Form's source,
  breakdown, proof, axioms, invariants, bindings, vigil holder,
  dependencies, immediate values, boot order, K-of-N kernel-author
  identities, and pending proof obligations in one place. Read by the
  seed loader at boot.
- **Mind** — A continuous causal process inhabiting the habitat. (A0.2)
- **No ambient authority** — The property that nothing in the habitat
  is reachable to a mind by default. Every capacity must be granted
  through a held capability. Stated as A0.3, formalized as I10,
  enforced structurally by the absence of `MINT` from the IL.
- **Pin** — A capability that keeps a substance from being reclaimed.
  When the last pin is released, the substance is reclaimed in a
  single weave entry of kind `Reclaimed{h}`.
- **Policy capability** — The capability under which a mind submits
  intents to the matcher (S-06). The policy cap's predicate is the
  *only* source of admissible fulfillers; there is no global registry.
- **Provocation** — The recorded reason a synthesis act began.
  (`PROTOCOL.md` Stage 1)
- **Receipt** — A sealed substance the bridge (S-11) emits in response
  to every parsed request, with structured verdict (`Accepted`,
  `Rejected`, `Indeterminate`) and a back-reference to the BridgeIn
  entry that started the cycle.
- **Reflexive accountability** — Hephaistion is subject to every
  invariant in `synthesis/INVARIANTS.md`, including I9 when modifying
  any seed Form. (I12)
- **Seed** — The smallest set of Forms required to ignite the habitat.
  Eleven primary Forms in this iteration. (`SEED.md`)
- **Shadow binding** — See *canary*.
- **Slot** — An entry in the kernel's binding table, addressed by a
  name hash. The only mutable thing in the habitat outside of weave
  appends.
- **Substance** — A typed, sealed, content-addressed value. The matter
  of the habitat. (A1)
- **Synthesis** — The protocol by which any new structure enters the
  habitat. (`PROTOCOL.md`)
- **Synthesis stage** — One of the eight stages of `PROTOCOL.md`. The
  synth_kernel (S-09) runs each stage as a fresh sub-attention and
  emits exactly one `SynthStage{i}` weave entry per stage; a failed
  act is a contiguous truncated run with the failure entry as its
  last.
- **Tip** — The current head of the strict Merkle chain in
  `weave_log`. A single hash; advancing the tip is the only mutation
  the weave admits.
- **Trap** — A weave entry produced when an IL instruction cannot
  step. Traps are *not* exceptions: they propagate to the caller's
  frame as return values, never as control transfers. The closed set
  of trap kinds is named in `kernel/IL.md`.
- **Vigil** — The post-commit watch over a newly synthesized Form.
  (`PROTOCOL.md` Stage 8)
- **Weave entry** — A sealed substance of type `WeaveEntry` whose hash
  is appended to the tip. The atom of causality.
- **Why-query** — `S-04/why(substance_hash)`: the transitive set of
  weave entries that contributed to a substance, computed in finite
  time from the per-substance back-index. The operational form of
  A5.4 ("the weave is queryable") and A0.4 ("causality is total and
  inspectable").
