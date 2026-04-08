# S-08 proof_checker — inspection record (DRAFT)

This document is the discharge artifact for S-08 obligation 2
("hand inspection of the kernel"). It is a structured line-by-
line review of every artifact that contributes to the proof
checker's trust surface, signed by the kernel-author identities
listed in `kernel/manifest.json` under `kernel_authors.identities`.

**Status: DRAFT.** The structure of the review is in place; the
actual signatures are placeholders, identical in shape to the
placeholder Ed25519 keys in the manifest. Both must be replaced
with real signatures by real kernel-author identities before
any actual ignition.

## What this document is

S-08 cannot prove itself in the strong sense (the bootstrap
exception). The breakdown specifies three layered discharges:

1. **Ground reflexive acceptance** — discharged structurally
   in `kernel/forms/S-08-proof-checker.proof` via the
   `WitnessExec` rule. *Necessary, not sufficient.*
2. **Hand inspection of the kernel** — *this document*.
3. **K-of-N multi-kernel consensus on replacement** —
   discharged at S-08 replacement time, vacuously satisfied
   for the pre-replacement lifetime.

The inspection record's job is to make discharge #2 a *real
artifact* rather than a slogan. The kernel-author identities
read every line of the artifacts listed below, sign their
review, and the seed loader checks the signatures against the
manifest before binding S-08.

## What is reviewed

A reviewer signing this document is asserting that they have
read each of the following artifacts in full and find them
faithful to their stated purpose:

### 1. The IL specification (`kernel/IL.md`)

- 34 opcodes, total small-step semantics
- Closed value-type set
- Closed trap-kind enumeration
- Absence of `MINT` (the I10 exception lives only in S-01)
- Absence of `TIME`, `NOW`, `RAND`, `MALLOC`, `SYSCALL`,
  `IMPORT`, `THROW`, `CATCH`
- Per-opcode rule producing exactly one of `step` / `trap` /
  `yield`

**Reviewer checklist:**

- [ ] I have read every opcode's rule and verified totality.
- [ ] I have verified `MINT` is absent.
- [ ] I have verified the trap-kind enumeration is closed.

### 2. The proof term language (`kernel/PROOF.md`)

- 12 sorts
- 17 term constructors
- 30-rule table (29 standard + `ExternalDischarge`)
- Closed admissibility for `WitnessExec` (S-07 #6, S-08 #1,
  S-09 #8 — and only those)
- Closed admissibility for `ExternalDischarge` (S-08 #2 and
  S-08 #3 — and only those)

**Reviewer checklist:**

- [ ] I have read every rule and verified its conclusion-
      derivation function is the natural-deduction shape it
      claims to be.
- [ ] I have verified that `WitnessExec` cannot be used outside
      its three admissible obligations.
- [ ] I have verified that `ExternalDischarge` cannot be used
      outside its two admissible obligations.

### 3. The proof checker Form (`kernel/forms/S-08-proof-checker.form`)

- Single export: `check`
- `:declared-caps (S-03/ROOT_RO)` — read-only, nothing else
- `:declared-traps ()` — totally — every failure is a return
  value
- The walker (`walker_visit`) split out as a sibling helper
  Form so the recursive piece is reviewable on its own
- The fixed rule table looked up by `rule_id` at the first
  step of every node
- `equiv_under_canon` calls for term equality at every leaf

**Reviewer checklist:**

- [ ] I have verified that `:declared-caps` is exactly
      `(S-03/ROOT_RO)` and contains no write-shaped cap.
- [ ] I have verified that `:declared-traps` is empty.
- [ ] I have read every line of `check`'s body and verified
      that it returns `Reject` rather than trapping on every
      failure path.
- [ ] I have read every line of `walker_visit`'s body and
      verified the recursive descent is well-founded.
- [ ] I have verified that the rule-id lookup against
      `RULE_TABLE_HASH` is the first gate of the walker, and
      that any `rule_id` not in the table returns
      `REJECT_UNKNOWN_RULE`.

### 4. The shared canonicaliser (`kernel/forms/helpers/canon-normalise.form`)

- Four passes: parse → opcode_fold → sort_blocks → emit
- `:declared-caps (S-03/ROOT_RO)`
- `:declared-traps (ETYPE)` — only on parse failure
- `equiv_under_canon` wrapper composing two `normalise` calls
  with a hash equality

**Reviewer checklist:**

- [ ] I have verified that `normalise` performs no mutation
      and no APPEND.
- [ ] I have verified that two source forms differing only in
      non-canonical features (whitespace, comments, opcode
      encoding, block ordering) produce byte-identical output.
- [ ] I have verified that `equiv_under_canon` decides
      structural equality through the canonicaliser, not
      through pointer comparison.

### 5. The abstract-model lemma library

The lemma library is sealed under `LEMMA_LIBRARY_HASH` (a
placeholder in the current manifest, to be resolved by the
build process). The source document is `kernel/lemma-library.md`,
which collects all 105 lemma entries (98 `LemmaApp` heads
referenced across the proof artifacts plus 7 implicit
composition lemmas). Each lemma's discharge is "look at the
body of the named Form at the named lines".

**Reviewer checklist:**

- [ ] I have read every entry in `kernel/lemma-library.md` (all
      105 lemmas across 14 source groups) and verified that
      each lemma corresponds to a structural fact about an
      encoded Form's body or about the IL specification.
- [ ] I have verified that no lemma in the library makes a
      claim that cannot be discharged by reading at most one
      Form's body and the IL specification.
- [ ] I have verified that the lemma library is *closed*: no
      lemma is defined in terms of another lemma whose own
      discharge would require additional structural reading
      not declared in this document.
- [ ] For each lemma whose source is "the helper Form when
      encoded", I have either verified the encoded helper or
      noted that the lemma is provisional pending the
      helper's encoding.

### 6. The manifest (`kernel/manifest.json`)

- The `kernel_authors.identities` array containing the
  identities authorised to sign this document
- The `K` value (currently 3)
- The `forms[S-08-proof-checker]` entry with
  `bootstrap_locked: true`
- The `proof_obligations_status` array's S-08 entry pointing
  at this inspection record

**Reviewer checklist:**

- [ ] I have verified that `kernel_authors.identities` contains
      *my* identity and that the public key recorded matches
      mine.
- [ ] I have verified that `K` is the value I agreed to.
- [ ] I have verified that S-08's `artifact` field points at
      `kernel/forms/S-08-proof-checker.proof` and its
      `inspection_record` field points at this document.

## Signature blocks

Each kernel-author identity signs by:

1. Reading every artifact above in full.
2. Marking every checklist item with their identity tag.
3. Producing an Ed25519 signature over the BLAKE3 hash of
   this document's canonical bytes (after stripping the
   signature blocks themselves and replacing them with the
   canonical empty form).
4. Appending the signature to the corresponding signature
   block below.

The seed loader, before binding S-08, verifies that the
signature blocks contain at least K valid signatures from
identities listed in the manifest. Fewer than K valid
signatures means S-08 cannot be bound and the seed cannot
ignite.

### kernel-author-0 (PLACEHOLDER)

- Identity: `kernel-author-0`
- Public key: `$$ED25519$$/placeholder-0`
- Reading completed: `[NOT YET]`
- Signature: `[NOT YET]`
- Notes: placeholder identity. Replace before ignition.

### kernel-author-1 (PLACEHOLDER)

- Identity: `kernel-author-1`
- Public key: `$$ED25519$$/placeholder-1`
- Reading completed: `[NOT YET]`
- Signature: `[NOT YET]`
- Notes: placeholder identity. Replace before ignition.

### kernel-author-2 (PLACEHOLDER)

- Identity: `kernel-author-2`
- Public key: `$$ED25519$$/placeholder-2`
- Reading completed: `[NOT YET]`
- Signature: `[NOT YET]`
- Notes: placeholder identity. Replace before ignition.

## Re-inspection requirements

This inspection record is invalidated and must be re-signed
whenever any of the following changes:

- `kernel/IL.md` (any change to the opcode table, value type
  set, or trap kind enumeration)
- `kernel/PROOF.md` (any change to the rule table, term
  language, or admissibility constraints on `WitnessExec` /
  `ExternalDischarge`)
- `kernel/forms/S-08-proof-checker.form`
- `kernel/forms/helpers/canon-normalise.form`
- The abstract-model lemma library content (the substance
  sealed under `LEMMA_LIBRARY_HASH`)
- The `kernel_authors.identities` array in the manifest (any
  add, remove, or key rotation)
- The `K` consensus parameter (any change requires a synthesis
  act against the manifest itself, which is not currently a
  protected operation but should become one post-v0.1.0)

A re-inspection produces a new version of this document; the
old version remains in the substance store as part of the
weave (per A0.4 — causality is total and inspectable).

## What this document does *not* certify

- The correctness of any proof artifact other than S-08's own
  reflexive case. The other ten Forms' proofs are checked by
  S-08 itself once S-08 is bound; the inspection record's job
  is to make *that* check trustworthy.
- The correctness of the abstract-model lemma library's
  *content*. The reviewer asserts that each lemma is a
  faithful structural reading; they do not re-derive every
  lemma from first principles. The smallness discipline of
  the lemma library is what makes this credible: if the
  library grows beyond what a reviewer can hold in their head,
  the discipline has failed and a new synthesis act against
  S-08 is required.
- The correctness of the seed loader's pre-binding sequence.
  That is a separate trust surface, post-v0.1.0.

## Why this matters

The seed cannot self-host until S-08 has been verified outside
the seed. The inspection record is the operational form of
"verified outside the seed". It is the place where human
reasoning takes responsibility for the part of the proof load
that the proof checker cannot bear about itself.

Without it, S-08 obligation 2 is a slogan. With it — even in
draft form, even with placeholder signatures — the slogan
becomes a structured commitment with named identities, named
checklists, named re-inspection triggers, and a named storage
location in the substance store.

The draft is the v0.1.0 deliverable. The signed final is the
last prerequisite for any actual ignition attempt.
