# Ignition bootstrap protocol

The operational form of A9 (Ignition Substrate). This document
specifies what `ignis0` — the stage-0 interpreter — must do to
bring the habitat from "a directory of files on disk" to "a
running habitat with hephaistion's first epoch firing". It is
the concrete counterpart to A9's abstract claim that a stage-0
interpreter exists.

This is not a specification for IgnisSynth. It is a specification
for the external software artifact that IgnisSynth depends on
at its base. `ignis0` lives in its own repository with its own
tests and its own authors. This document is the *contract* that
repository must satisfy.

## Naming and versioning

- **`ignis0`** is the informal name; any stage-0 implementation
  that satisfies this document qualifies, regardless of its
  language or repository.
- Stage-0 implementations declare their version as
  `ignis0/{language}/{semver}`, e.g. `ignis0/rust/0.1.0`.
- The IL specification version this document targets is
  `kernel/IL.md` at the hash recorded in `kernel/manifest.json`.
  A stage-0 built against an older IL.md version is *not*
  eligible to ignite a seed built against a newer version.

## Contract: what `ignis0` must implement

`ignis0` must be able to execute programs in the IL specified
by `kernel/IL.md`. Concretely:

### 1. Parse Form wire bytes

Given a byte sequence claimed to be a canonical Form, produce
the structured `ParsedForm` record defined by `kernel/IL.md §
Encoding`. This is the same operation `S-07/parse_form` performs
inside the habitat; `ignis0` must do it from outside, using its
host language's parser tools.

### 2. Execute IL instructions

For every opcode in `kernel/IL.md`, implement the small-step
rule given in the opcode table. The rule is total: every
well-typed `ExecState` must produce exactly one of
`step(s')` / `trap{kind}` / `yield(continuation)`.

### 3. Read and write substances

`ignis0` must implement the substance store operations defined
by `kernel/forms/S-03-substance-store.form` at the abstract
level: `seal`, `read`, `pin`, `unpin`, `digest`. `ignis0`'s
implementation is a persistent hash trie written in the host
language, not a Form. The guarantee is that the abstract
behaviour is bit-compatible with S-03's spec.

### 4. Append to the weave

Same shape: `ignis0` implements `S-04/append` in the host
language, preserving strict Merkle-chain semantics and
per-substance back-indexing.

### 5. Allocate attention

Same shape: `ignis0` implements `S-05/attention_alloc` in the
host language, preserving budget conservation and the
(deadline, id) ordering.

### 6. Verify capabilities

Same shape: `ignis0` implements `S-02/cap_registry` in the host
language.

### 7. Honor I7

`ignis0` must not introduce non-determinism. Two runs of the
same sealed seed with the same inputs must produce byte-
identical weave tails. `ignis0` implementations in languages
with non-deterministic features (concurrent execution, reliance
on system time, non-deterministic hash maps) must disable those
features during ignition.

### 8. Report substrate events

`ignis0` may encounter conditions the IL cannot express:
out-of-memory, power loss, hardware fault, host-language panic.
When such an event interrupts execution, `ignis0` must record
a `SubstrateEvent` weave entry (a new entry kind the seed loader
introduces at binding time) describing the event, and then
halt. The habitat post-mortem inspects these entries when it
is re-ignited.

## Ignition sequence

The sequence `ignis0` follows to bring a seed from disk to
running habitat. Each step is named and has explicit
preconditions, postconditions, and failure modes.

### Step 0: Check input integrity

- **Input**: a directory tree rooted at the seed repository.
- **Action**: verify the directory tree's top-level structure
  matches `kernel/manifest.json`'s declared layout. Every file
  `kernel/manifest.json` references must exist.
- **Failure mode**: missing files halt `ignis0` with exit code
  `E_SEED_INCOMPLETE`. No weave is started.

### Step 1: Seal the cold substances

- **Input**: the seed repository's file tree.
- **Action**: `ignis0` walks the manifest's `forms`, `axioms`,
  `invariants`, `protocol`, `il`, `proof_obligations_status`,
  and breakdown references in dependency order (declared in
  `manifest.json`'s `boot_order` field). For each file, it
  canonicalises the bytes (stripping whitespace and comments
  per `kernel/forms/helpers/canon-normalise.form`'s spec),
  seals them through its host-language S-03 implementation,
  and records the resulting hash.
- **Output**: a mapping from each `$$BLAKE3$$` placeholder in
  `kernel/manifest.json` to the actual resolved hash. This
  mapping is the **resolved manifest**, and `ignis0` writes it
  to a sibling file `kernel/manifest.resolved.json`.
- **Failure mode**: any canonicalisation or sealing error halts
  with exit code `E_SEED_CANON_FAIL`. No weave is started.

### Step 2: Run the ignition fixed-point check (A9.3)

- **Input**: the resolved manifest from Step 1.
- **Action**: `ignis0` executes the canonical fixed-point check
  described below.
- **Output**: pass or fail.
- **Failure mode**: fail halts with exit code `E_FIXED_POINT_FAIL`.
  No weave is started. The seed repository is unchanged; a
  subsequent ignition attempt with a different `ignis0` (or a
  different seed) may succeed.

#### The canonical fixed-point check

Let `F` be the following minimal Form, sealed at build time
into the seed's substance store under a known hash
`$$BLAKE3$$/ignition/fixed-point-F`:

```
(form
  :type-tag      "Form/v1"
  :name          "ignition_fixed_point_F"
  :declared-caps ()
  :declared-traps ()
  :arity         1
  :locals-n      1
  :code (
    STORE   0
    LOAD    0
    PUSH    1
    ADD
    RET
  )
  :rationale-hash $$BLAKE3$$/ignition/fixed-point-F-rationale
)
```

`F` takes one Nat on the stack, adds 1 to it, returns the
result. Chosen for three reasons:
- It has zero cap requirements (so no cap infrastructure is
  needed to run it).
- It has zero yields (so no attention-allocation infrastructure
  is exercised).
- It exercises exactly three IL opcodes (`STORE`, `LOAD`, `PUSH`,
  `ADD`, `RET`) which between them require every major
  primitive category to be wired up in `ignis0`.

The canonical input is `42`.

The check runs three computations:

1. **Direct**: `ignis0` evaluates `F`'s body directly on `42`.
   Expected output: `43`.
2. **One-level indirect**: `ignis0` evaluates S-07's `execute`
   Form, passing `(hash(F), new_attention(), [42])`. S-07's
   body is IL; `ignis0` interprets it; S-07 in turn interprets
   `F`. Expected output: `43`.
3. **Two-level indirect**: `ignis0` evaluates S-07's `execute`
   Form with `(hash(S-07), new_attention(), [hash(F), new_attention(),
   [42]])`. Now S-07 is interpreting S-07 which is interpreting
   `F`. Expected output: `43`.

All three must produce `43`. If any differs, `ignis0` is not
faithful to the IL on this canonical case, and the seed halts.

The one-level indirect case proves `ignis0` can interpret the
runtime. The two-level indirect case proves the runtime can
interpret itself under `ignis0`. Together they establish the
necessary condition named in A9.3.

### Step 3: Bind the primary Forms

- **Input**: resolved manifest from Step 1, pass verdict from
  Step 2.
- **Action**: for each Form in `boot_order`, `ignis0` binds the
  slot `{name}` to the Form's resolved hash in the host-language
  substance store. The order matters: helper Forms are bound
  before primary Forms that depend on them, so every `READSLOT`
  call during subsequent execution resolves to an encoded
  target.
- **Output**: a fully-bound substance store with every slot
  declared in `kernel/manifest.json` resolved.
- **Failure mode**: a missing helper binding (e.g., an encoded
  primary Form references a slot whose helper is still catalogued
  as pending in STUBS.md) halts with exit code
  `E_UNBOUND_HELPER`, naming the specific unbound slot.

### Step 4: Verify proof artifacts

- **Input**: bound substance store.
- **Action**: for each entry in
  `proof_obligations_status` whose `status` is `discharged` or
  `discharged_structural`, `ignis0` invokes S-08's `check` Form
  (now bound) on the proof's rule tree and the declared claim.
  A Reject verdict halts ignition with exit code
  `E_PROOF_REJECT`, naming the failing obligation.
- **Output**: a proof verdict substance for each primary Form,
  sealed and referenced from the manifest's
  `proof_verification_results` field (produced at ignition
  time, not at build time).
- **Failure mode**: any Reject halts. Additionally, any
  `pending_leaf` entry whose leaf does not resolve at this
  stage halts with `E_LEAF_UNRESOLVED`.

### Step 5: Check the S-08 inspection record signatures

- **Input**: the resolved hash of
  `kernel/forms/S-08-proof-checker.inspection-record.md`.
- **Action**: `ignis0` reads the signature blocks at the end of
  the inspection record, verifies each Ed25519 signature against
  the corresponding public key in
  `manifest.json#/kernel_authors/identities`, and counts how
  many signatures are valid.
- **Output**: a count `K_valid`.
- **Failure mode**: if `K_valid < manifest.kernel_authors.K`
  (currently 3), ignition halts with exit code
  `E_INSPECTION_K_SHORT`. This is the only place the seed
  acknowledges real humans: if the kernel-author identities
  have not signed the inspection record, the seed refuses to
  bind S-08 and therefore refuses to ignite.

### Step 6: Execute S-01 (ignite)

- **Input**: the fully-verified, fully-bound substance store.
- **Action**: `ignis0` invokes `S-01/ignite`'s execute function
  with no arguments. S-01's body:
  1. Mints the root capability `R`.
  2. Appends `E0` (the genesis `Synthesized` weave entry).
  3. Self-erases by overwriting the `ignite` slot with
     `IGNITED`.
- **Output**: the root capability `R`.
- **Failure mode**: S-01 trapping (which would be a seed
  inconsistency, not an `ignis0` bug) halts with exit code
  `E_GENESIS_TRAP`.

### Step 7: Seed-loader hand-off

- **Input**: the root capability `R`.
- **Action**: `ignis0` hands `R` to the bootstrapping mind (an
  attention allocated at ignition with the declared root
  budget), which attenuates `R` into the four ignition caps
  for hephaistion (`read_weave`, `read_metrics`,
  `propose_synthesis`, `mutate_kernel_gated`), the bridge's
  policy cap, and the seed-loader's own mutation cap.
- **Output**: hephaistion's root attention, with budget =
  `HEPHAISTION_EPOCH_BUDGET`.

### Step 8: Cold weave import

- **Input**: bootstrapping mind's authority + the breakdowns.
- **Action**: the bootstrapping mind imports every breakdown
  file as a `Synthesized` weave entry, in dependency order,
  with each entry's `grounding` field citing the axioms
  declared in the breakdown and its `rationale_hash` pointing
  at the breakdown substance's hash.
- **Output**: a weave whose first ~12 entries (genesis + 11
  breakdowns) constitute the habitat's prehistory.

### Step 9: Drop root

- **Input**: hephaistion's root attention is allocated, the
  bridge is ready.
- **Action**: the bootstrapping mind revokes its own root
  capability and dissolves. By S-02 obligation 3 (revocation
  totality), every descendant of the root cap is now dead
  except the four caps explicitly granted to hephaistion and
  the bridge.
- **Output**: no mind holds the root capability. The I10
  closure of authority is now the ordinary inductive case
  (S-01 discharged the base case at Step 6).

### Step 10: Begin

- **Input**: everything.
- **Action**: `ignis0`'s main loop switches modes. It stops
  being "the seed loader" and starts being "the IL interpreter
  service". Hephaistion's first epoch fires via `S-05/tick`.
  The bridge's inbound endpoint begins accepting requests.
- **Output**: a running habitat.

From Step 10 onward, `ignis0` runs in the background as the
substrate. Every subsequent act of the habitat is interpreted
by `ignis0`. The habitat itself does not know `ignis0` is
there, except through the `SubstrateEvent` entries `ignis0`
posts when something it cannot handle occurs.

## Reference stage-0 status

As of `v0.1.0-pre-ignition`: no reference `ignis0` exists.
This document specifies the contract; the implementation is a
post-v0.1.0 deliverable with its own milestone schedule in a
separate repository.

The recommended first `ignis0` is written in Rust because:
- Rust has strong determinism guarantees (no uninitialised
  memory reads, predictable panic semantics).
- Rust has well-maintained BLAKE3 and Ed25519 implementations.
- Rust's type system can express the substance-type layouts
  in `kernel/types/SCHEMA.md` directly.
- The IgnisSynth project has no preference for Rust over any
  other language; Rust is named here as a practical suggestion
  and nothing in this document binds stage-0 to it.

A stage-0 implementation in Python or OCaml or Zig is equally
acceptable. What is not acceptable is a stage-0 that depends
on non-deterministic features of its host language, or one
that cannot produce the resolved manifest in Step 1, or one
that fails the canonical fixed-point check in Step 2.

## What stage-0 does *not* do

- **Stage-0 does not synthesise.** It cannot produce new Forms
  from axioms, because it does not implement the synthesis
  protocol. Synthesis happens inside the habitat, through S-09.
- **Stage-0 does not vigil.** Vigils are held by minds, and
  stage-0 is not a mind.
- **Stage-0 does not prove.** It invokes S-08 during Step 4,
  but S-08 does the proof-checking; stage-0 just orchestrates
  the invocations.
- **Stage-0 does not upgrade itself.** Post-ignition, a
  Hephaistion synthesis act may produce a new stage-0
  (perhaps by compiling the IL to native code for a specific
  architecture), but replacing the running stage-0 with the
  new one requires a checkpoint-and-restart sequence outside
  the habitat, not a synthesis act.
- **Stage-0 does not persist state across ignitions.** Every
  ignition is a fresh run. The seed loader reads the seed
  repository from disk each time, re-resolves hashes, re-runs
  the fixed-point check. If the seed repository is unchanged,
  the resolved hashes and the weave tail are byte-identical
  under I7.

## Why this works

The classical objection to self-hosting runtimes is "you cannot
bootstrap the compiler from itself". The classical resolution
is the one this document takes: **bootstrap from a prior thing**.
The prior thing is stage-0, written in a language that already
runs. This is not cheating; it is what every operating system
has ever done. Unix was first compiled by a compiler that was
not Unix. GCC's first version was not compiled by GCC. The
difference is that IgnisSynth names the stage-0 explicitly,
gives it a contract, and verifies the contract at every
ignition.

The fixed-point check (A9.3) does something stronger than the
classical bootstrap: it observes that stage-0's direct
interpretation of `F` agrees with stage-0's interpretation of
S-07 interpreting `F`. If they disagree, stage-0 is broken *on
the specific case of interpreting the habitat's own runtime*,
which is the only case the habitat actually cares about.
Correctness on other cases is useful but not required for
ignition.

The Gödelian objection — "you still need to trust stage-0" —
is true and unavoidable. But it is the *same* trust a program
places in its CPU. The habitat does not pretend otherwise.
A9.5 names stage-0 as the substrate and refuses to grant it
any authority within the habitat's own discipline. The
habitat's discipline is self-consistent *given* a faithful
substrate, and the fixed-point check makes "faithful
substrate" observable at ignition time.

This is as solid a foundation as any real system has ever
had. The alternative — a self-hosting interpreter that needs
no prior infrastructure — is not an alternative. It does not
exist. Any attempt to build one eventually admits a prior
stage, and the question is only whether the admission is
explicit.

IgnisSynth makes it explicit.
