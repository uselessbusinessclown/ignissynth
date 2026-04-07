# Seed Synthesis Tasks

This document hands the production of the seed (`synthesis/SEED.md`)
out to implementing AI agents. Each task is a single Form. Each task
is a synthesis act subject to `synthesis/PROTOCOL.md`. Each task
produces, alongside its Form, the rationale, the self-test set, the
declared invariants, the simulation record, and (where required) the
proof.

The tasks are ordered by dependency. They are intentionally written so
that an autonomous synthesizing agent can pick exactly one task and
complete it without consulting any other implementer.

## Task index

| ID    | Form              | Cites axioms     | Depends on        | Proof?|
|-------|-------------------|------------------|-------------------|-------|
| S-01  | `ignite`          | A0.7, A2.1       | —                 | yes   |
| S-02  | `cap_registry`    | A2.*             | —                 | yes   |
| S-03  | `substance_store` | A1.*, A0.5       | —                 | yes   |
| S-04  | `weave_log`       | A5.*             | S-03              | yes   |
| S-05  | `attention_alloc` | A3.*, A0.5       | S-02, S-04        | yes   |
| S-06  | `intent_match`    | A4.*, A2.5       | S-02, S-05        | yes   |
| S-07  | `form_runtime`    | A6.*             | S-03, S-05        | yes   |
| S-08  | `proof_checker`   | A0.4, A6.5, A7.3 | S-03              | bootstrap |
| S-09  | `synth_kernel`    | A0.7, A7.*       | S-04, S-07, S-08  | yes   |
| S-10  | `hephaistion_seed`| A7, A8           | S-09              | yes   |
| S-11  | `bridge_proto`    | A8.*             | S-06              | yes   |

## How to take a task

1. Copy `breakdown/_TEMPLATE.md` to `breakdown/S-XX-{name}.md`.
2. Fill in the Provocation, Grounding, Candidate(s), Self-tests,
   Rationale, and Proof Sketch sections.
3. Open a synthesis session with `synthesis/PROTOCOL.md` open in
   front of you. Do not skip stages.
4. Produce the Form in `kernel/forms/S-XX-{name}.form` (a sealed,
   typed substance — until `form_runtime` exists, this is a textual
   intermediate that the seed compiler accepts).
5. Run the Stage 4 simulation harness against the candidate.
6. When the candidate is complete, record the trial and propose the
   commit. The commit is a git commit referencing your Form hash and
   your synthesis chain.

## Conventions for synthesizing agents

- **No prior art justification.** Your rationale must derive from the
  axioms. "This is how Linux does it" is never an admissible reason.
  "This is how seL4 does it" is never an admissible reason. You may
  *converge* on a known idea; you may not *cite* it as authority.
- **Diverse candidates.** Produce at least two structurally different
  candidates per Form, unless the task explicitly says one is
  sufficient. Record why you chose the one you chose.
- **Smallest sufficient.** Prefer the smallest Form that satisfies the
  invariants. Do not add features for the future. The future will
  synthesize them itself, under its own provocation.
- **Speak to the weave.** Your rationale and your simulation record
  are not "documentation". They are the *primary artifact* of the
  task. The Form is just the part that runs.
