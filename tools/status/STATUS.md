# IgnisSynth status dashboard

> **Generated artifact.** Do not edit by hand. Regenerate with:
> `bash tools/status/build-status.sh`
>
> Generated at: `2026-04-22T07:14:41Z`
> From commit:  `8838b20`
>
> This page is the single source for repo status numbers. Other
> docs (READMEs, ROADMAP narrative) should link here rather than
> restate counts that drift.

---

## Versions

| Component | Version |
|---|---|
| Seed     | `0.1.0-pre-ignition` |
| ignis0   | `0.3.0-compute` (MSRV `1.75`) |

## Primary Forms

| Metric | Value |
|---|---|
| Expected primary Forms | 11 |
| Forms present          | 11 |
| Forms with proof       | 11 |

### Proofs (per-file obligation counts and declared verdicts)

| Form | Obligations | Declared verdict |
|---|---:|---|
| `S-01-ignite` | 7 | Pass |
| `S-02-cap-registry` | 8 | Pass |
| `S-03-substance-store` | 6 | Pass |
| `S-04-weave-log` | 6 | _(implicit / not declared)_ |
| `S-05-attention-alloc` | 6 | Pass |
| `S-06-intent-match` | 6 | Pass |
| `S-07-form-runtime` | 6 | _(implicit / not declared)_ |
| `S-08-proof-checker` | 3 | Structural |
| `S-09-synth-kernel` | 8 | _(implicit / not declared)_ |
| `S-10-hephaistion-seed` | 8 | Pass |
| `S-11-bridge-proto` | 6 | _(implicit / not declared)_ |

**Total obligations across all proofs:** 70
**Verdicts:** 6 Pass · 1 Structural · 4 unspecified

## Helper Forms

| Metric | Value |
|---|---:|
| Helper `.form` files | 10 |
| Helpers encoded (per `STUBS.md`) | 86 |
| Helpers pending (per `STUBS.md`) | 99 |

## IL opcode count

| Source | Value |
|---|---:|
| Implementation (`ignis0/src/opcode.rs` `Opcode` enum) | 35 |
| Specification (`kernel/IL.md` declared string) | 35 (`Thirty-five exactly`) |
| In sync | true |

## Manifest integrity

| Metric | Value |
|---|---|
| Required keys present | yes ✓ |
| Forms in manifest         | 11 |
| Axioms in manifest        | 9 |
| Kernel authors            | 3 |

## Axioms

| Metric | Value |
|---|---:|
| Axiom files on disk          | 10 |
| Axiom entries in manifest    | 9 |
| Files not in manifest        | `A9-ignition-substrate` |

## Invariants

Total invariants in `synthesis/INVARIANTS.md`: **12**

## ignis0 milestone track

| Tag | Status |
|---|---|
| `v0.2.0-ignition` | ✓ done |
| `v0.2.1-ignis0-call` | ✓ done (`c4c033a`) |
| `v0.2.2-ignis0-wire` | ✓ done (`8353185` + post-merge iteration) |
| `v0.2.3-ignis0-fp` | ✓ done |
| `v0.2.4-ignis0-cap` | ✓ done |
| `v0.2.5-ignis0-store` | depends on Trie.md |
| `v0.3.0-compute` | ✓ done (`d28b466`) — landed out of order; schedule above is corrected |
| `v0.3.0-envelope+ci` | ✓ done (`e954a27`) |
| `v0.3.0-build-int` | ✓ done (`a130590`) |

**Milestone summary:** 8 done · 1 blocked · 0 other

## Drift detected

The dashboard noticed the following inconsistencies. None of these
block the build by themselves; CI decides which warrant a hard fail.

- proofs without explicit '; Verdict:' line: S-04-weave-log, S-07-form-runtime, S-09-synth-kernel, S-11-bridge-proto
- axioms on disk but not in manifest: A9-ignition-substrate

