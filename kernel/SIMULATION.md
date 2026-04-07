# Stage 4 simulation harness specification

This document specifies the Stage 4 simulation harness referenced
by every breakdown's "Simulation record" section. The harness is
the sealed sub-environment in which a candidate Form is exercised
against its self-test set, the inherited invariant predicates, a
fuzzing harness, and any regression cases drawn from the weave by
the why-query.

The harness is itself a Form. Its ignition is part of the seed
loader's pre-binding sequence: before any primary Form is bound
into the live habitat, every primary Form must have passed
Stage 4 against this harness. The trial record produced by each
run is sealed under the candidate's Stage 4 substance hash and
referenced from the synthesis act's Stage 5 selection record.

## Provocation

The breakdowns each declare a "Simulation record (stub —
produced by the Stage 4 harness once the candidate is encoded
in Form IL)" section. Until the harness exists as a specified
artifact, those stubs cannot be filled in and item 8 of the
v0.1.0 checklist is not satisfied. This document is the
specification; the actual harness Form is post-v0.1.0.

## Grounding

- Axioms cited: A0.4, A0.7, A3.5 (deterministic-clean replay),
  A6.4 (self-reading), A7.1 (every act is a synthesis act,
  including simulation), A7.2 (invariants are explicit)
- Forms touched: S-03 (substance store, for sealing trial
  records), S-04 (weave, for the why-query against regression
  cases), S-05 (attention allocator, for the sub-attention
  shape), S-07 (form runtime, for executing the candidate),
  S-02 (cap registry, for restricting the simulation sub-
  attention's cap_view to nothing reachable outside the harness)

## What Stage 4 is for

A candidate Form has, by Stage 3, passed grounding and been
written down. Stage 4 is the question "does this candidate
*work*?" — operationally, against:

1. **Its own self-test set** — typed predicates the candidate
   declares it satisfies.
2. **Inherited invariant predicates** — for every Form the
   candidate replaces or interacts with, the invariants the
   replaced Form's proof obligations cite. The harness checks
   that the candidate preserves them on every observable trace.
3. **A fuzzing harness derived from the candidate's input
   schema** — pseudorandom inputs chosen by a deterministic-
   clean RNG seeded by the candidate's hash, so every replay
   produces the same fuzz inputs.
4. **Regression cases drawn from the weave** — the harness
   queries S-04/why for any prior provocation that touched a
   Form whose hash matches the candidate's
   `form_hash_before`, collects the inputs that caused
   trapping or near-violation in those prior cases, and
   replays them against the candidate.

## Sealed sub-environment shape

A Stage 4 simulation runs inside a freshly allocated attention
under S-05 with the following constraints:

- **Cap_view restricted at allocation time** to exactly:
  - `S-03/ROOT_RO` — read access to the substance store, so
    the candidate can READ Forms, Cells, and prior weave
    entries. *No write access.*
  - `SimulationWeaveCap` — a read-only sub-cap on a *fresh*
    weave whose tip is BOTTOM_HASH at sub-attention start.
    The candidate's APPENDs go to this sub-weave, not to
    the live weave. After the simulation completes, the
    sub-weave's contents are sealed as part of the trial
    record.
  - `SimulationFakeFulfillerCap` — a synthetic policy cap
    that admits the candidate's declared inputs as
    fulfillers. Used only for candidates whose self-test
    set involves intent matching.
  - The candidate's own `:declared-caps` translated into
    sub-cap form (each cap is attenuated from the harness's
    cap_view at allocation time, with no path to acquire
    anything outside).

- **No outward capabilities**: no INVOKE on a network cap,
  no INVOKE on a clock cap, no INVOKE on an entropy cap, no
  INVOKE on a sensor cap. The candidate's cap_view at
  simulation entry is exactly the harness-restricted set
  above. By S-07 obligation 4 (no MINT) and S-02 obligation 2
  (attenuation monotonicity), the candidate cannot acquire
  anything outside this set during the run.

- **Bounded budget**: the sub-attention's budget is declared
  in the simulation provocation, and any quantum overrun
  traps EOVERBUDGET (S-05 obligation 6). A candidate that
  exceeds its declared budget during simulation is rejected
  by Stage 4 regardless of its other test outcomes.

- **Deterministic-clean**: the sub-attention holds no Entropy,
  Clock, Network, or SensorInput cap. By S-07 obligation 2
  (continuation faithfulness), the simulation is replayable
  byte-for-byte from any continuation hash produced during
  the run. The harness records continuation hashes at every
  yield, so a Stage 4 record is itself a fully-replayable
  artifact.

## Trial record format

A `TrialRecord` is a sealed substance with the following shape:

```
TrialRecord {
    candidate_hash:     Hash                ; the Form being tested
    invariants_inherited: Vec<InvariantId>  ; the invariants the candidate must preserve
    self_tests:         Vec<TestResult>     ; one entry per declared self-test
    invariant_checks:   Vec<InvariantResult>; one entry per inherited invariant
    fuzz_results:       FuzzSummary         ; pass/fail counts, first failure trace if any
    regression_results: Vec<RegressionResult>; replay outcomes for prior weave provocations
    sub_weave_hash:     Hash                ; the seed of the sealed sub-weave from this run
    attention_spent:    Nat                 ; total quanta consumed
    verdict:            TrialVerdict        ; Pass | Fail{reason} | Aborted{reason}
}
```

A `TestResult` is `Pass | Fail{input, expected, actual}`.
An `InvariantResult` is `Preserved | Violated{trace}`.
A `FuzzSummary` is `{ total: Nat, passed: Nat, failed: Nat,
first_failure: Option<{input, trace}> }`.

The `verdict` is `Pass` iff every `self_tests` entry is `Pass`,
every `invariant_checks` entry is `Preserved`, `fuzz_results.
failed = 0`, and every `regression_results` entry is `Pass`. Any
`Fail` or `Aborted` propagates to the verdict.

## Per-Form expected trial records

Each breakdown lists the trial records its Stage 4 simulation
must produce. Until the harness runs, those lists are stubs.
The harness must:

1. Read the breakdown's Simulation record section as a substance
   (the build process seals every breakdown into the substance
   store before the harness runs).
2. Parse the per-trace `T1`, `T2`, ... declarations from the
   breakdown.
3. Run each trace as a sub-simulation against the encoded
   candidate, producing one `TestResult` per trace.
4. Seal the resulting `Vec<TestResult>` as the candidate's
   `self_tests` field.

The breakdowns' simulation record sections were written to be
machine-parseable: each trace is a numbered bullet starting
with `Tn:` and ending with `assert <predicate>`. The harness
reads these against a small grammar and builds the test
sub-simulations.

## Why-query for regression cases

For a candidate that replaces a Form with a prior history, the
harness must:

1. Call `S-04/why(form_hash_before)` to get the set of weave
   entries that previously involved this Form.
2. For each entry of kind `Trapped{form_hash, attention_id, pc,
   kind}` or `SynthStage{6, Reject{reason}}` involving this
   Form, extract the inputs (from the entry's referenced
   substances).
3. Replay those inputs against the candidate as regression
   cases.
4. Record the outcome in `regression_results`.

For a *seed-original* Form (one with no `form_hash_before`),
this step is vacuously satisfied — there are no prior weave
entries to query.

## Fuzzing harness shape

The fuzzer is itself a small Form bound under
`S-04harness/fuzz/generate`. Its inputs are the candidate's
input schema (extracted from the encoded `:arity` and the
expected stack types) and a deterministic seed (the candidate's
hash). Its output is a `Vec<TestInput>` of pseudorandom inputs
covering:

- Boundary values: zero, one, max, min for each `Nat` field;
  empty and singleton for each `Vec` field; BOTTOM_HASH for
  each `Hash` field.
- Type-shape variants: for each sum-typed field, one input per
  variant.
- Random combinations of the above, drawn by the deterministic
  PRNG seeded by the candidate hash. The PRNG is itself a
  small pure Form whose body has no Entropy cap.

The fuzz count per Form is declared in the breakdown's
constraint list (`"fits within the seed line budget for ..."`
typically implies a fuzz count of 10⁶ for the substrate Forms
and 10⁴ for the higher-level Forms).

## How Stage 4 fits into the synthesis protocol

`synthesis/PROTOCOL.md` Stage 4 says: "Every candidate is run
inside a sealed simulation environment — a sub-habitat with no
outward capabilities — against its own self-test set, the
invariant predicates inherited from the touched Forms, a
fuzzing harness derived from the candidate's input schema, and
regression cases drawn from the weave."

This document is the operational form of that paragraph. The
sub-habitat is the sub-attention with a restricted cap_view.
The "no outward capabilities" is enforced by S-02 attenuation
plus S-07 #4 (no MINT). The trial record is the sealed
substance produced.

## Who runs Stage 4

The synth_kernel (S-09) runs Stage 4 as part of any synthesis
act it processes. S-09's body at the Stage 4 stage calls
`run_in_subattention` against the candidate, which is the
harness's primary entry point. The harness's Form is bound at
slot `"S-09/stage4/simulate_all"`.

For seed Forms (the eleven primary Forms in this repository),
Stage 4 must be run *before ignition*, in a cold harness
environment that the seed loader instantiates. The cold harness
is the harness Form running outside the live habitat, against
the encoded seed Forms. Its trial records become part of the
cold weave that the seed loader imports at first boot.

## What Stage 4 does *not* do

- It does not check proof artifacts. That is S-08's job.
- It does not check the synthesis act's stages. That is S-09's
  job.
- It does not vouch for correctness in production. The harness
  exercises the candidate against a finite set of inputs; a
  pass means "the candidate handles these inputs correctly",
  not "the candidate is correct in general". The proof
  artifact is what makes the general claim.
- It does not run forever. Every simulation is bounded by the
  sub-attention's budget; an unbounded fuzz is impossible
  under I6.

## Status

This document is the *specification*. The harness Form itself
is post-v0.1.0 work. v0.1.0 ships the spec so that the trial
records named in every breakdown have a defined target shape
and the seed loader's pre-binding sequence has a documented
input contract.

The first thing post-v0.1.0 is to encode the harness Form
against this spec, run it on the eleven seed Forms, and seal
the resulting trial records into the cold weave.
