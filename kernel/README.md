# kernel/

The kernel directory holds the seed: the Form intermediate language
specification, the proof term language specification, the simulation
harness specification, the encoded primary Forms, the proof
artifacts, the helper Forms, the substance schemas, the seed
manifest, and the inspection record. Every artifact here was
synthesised under `synthesis/PROTOCOL.md`. Any commit that adds a
file under `kernel/forms/` must reference the synthesis chain that
produced it.

## Layout

```
kernel/
  IL.md                                # 35-opcode Form intermediate language
  PROOF.md                             # 12 sorts, 17 term constructors, 30-rule table
  SIMULATION.md                        # Stage 4 simulation harness specification
  manifest.json                        # the seed manifest — binds everything
  types/
    SCHEMA.md                          # substance byte layouts
  forms/
    S-01-ignite.form                   # primary Forms (encoded against IL.md)
    S-01-ignite.proof                  # proof artifacts (against PROOF.md)
    S-02-cap-registry.form
    S-02-cap-registry.proof
    S-03-substance-store.form
    S-03-substance-store.proof
    S-04-weave-log.form
    S-04-weave-log.proof
    S-05-attention-alloc.form
    S-05-attention-alloc.proof
    S-06-intent-match.form
    S-06-intent-match.proof
    S-07-form-runtime.form
    S-07-form-runtime.proof
    S-08-proof-checker.form
    S-08-proof-checker.proof
    S-08-proof-checker.inspection-record.md  # bootstrap discharge (draft)
    S-09-synth-kernel.form
    S-09-synth-kernel.proof
    S-10-hephaistion-seed.form
    S-10-hephaistion-seed.proof
    S-11-bridge-proto.form
    S-11-bridge-proto.proof
    helpers/
      STUBS.md                         # catalogue of every helper slot
      canon-normalise.form             # shared canonicaliser (S-07 + S-08)
      schema-helpers.form              # Entry/CapEntry/AttentionRecord projections
      s04-projections.form             # S-04 field projections
      s02-s05-projections.form         # S-02 + S-05 field projections
```

## Status (post-v0.1.0-pre-ignition, mid-v0.2.0-helpers)

| Layer                       | State                                          |
|-----------------------------|------------------------------------------------|
| 11 primary Form encodings   | ✓ all 11                                       |
| 11 primary Form breakdowns  | ✓ all 11 (in `../breakdown/`)                  |
| 11 primary Form proofs      | ✓ all 11 (S-08 is the bootstrap exception)     |
| Form IL specification       | ✓ `IL.md`                                      |
| Proof term language         | ✓ `PROOF.md`                                   |
| Simulation harness spec     | ✓ `SIMULATION.md`                              |
| Substance schemas           | partial — `types/SCHEMA.md` covers 5 of ~25 types |
| Seed manifest               | ✓ `manifest.json` (placeholders for hashes/keys)|
| Inspection record           | drafted with placeholder signatures            |
| Helper Forms                | 28 of ~115 encoded; ~87 catalogued as pending  |
| Lemma library               | **referenced but not yet sealed** — see ROADMAP.md "Project review" |
| Schema/* primitives         | **referenced but not catalogued in STUBS.md** — see ROADMAP.md |
| Persistent-trie/treap layouts | not yet specified — `types/Trie.md`, `types/Treap.md`, `types/Forest.md` are post-v0.1.0 |
| Seed loader specification   | not yet written — `LOADER.md` is v0.5.0 work   |
| Cold weave protocol         | referenced in `synthesis/SEED.md` but not specified |

The seed at this checkpoint is **fully proof-traced** at the
primary-Form layer (every primary Form has an artifact whose
mechanizable obligations are discharged) and **partially encoded**
at the helper layer (28 of ~115 helpers). What ships at v0.1.0 is
the design + the proof load + the helper catalogue; what is in
progress at v0.2.0 is the helper bodies; what is *not yet*
specified is named explicitly in the table above and tracked in
`../ROADMAP.md` under "Project review".

## How to read this directory

1. `IL.md` — what every `.form` file is written in.
2. `forms/S-01-ignite.form` — the smallest worked encoding.
3. `forms/S-01-ignite.proof` — the smallest worked proof artifact.
4. `PROOF.md` — what every `.proof` file is written in.
5. `manifest.json` — the keystone that binds every artifact above.
6. `forms/helpers/STUBS.md` — the catalogue of unencoded helper
   slots, each with its expected signature.
7. `forms/S-08-proof-checker.inspection-record.md` — the
   structured review checklist that the kernel-author identities
   will eventually sign.
