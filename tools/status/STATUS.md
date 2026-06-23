# IgnisSynth status dashboard

> **Generated artifact.** Do not edit by hand. Regenerate with:
> `bash tools/status/build-status.sh`
>
> Generated at: `2026-06-23T21:31:40Z`
> From commit:  `52aedb7`
>
> This page is the single source for repo status numbers. Other
> docs (READMEs, ROADMAP narrative) should link here rather than
> restate counts that drift.

---

## Versions

| Component | Version |
|---|---|
| Seed     | `0.1.1-pre-ignition` |
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
| `S-04-weave-log` | 6 | Pass |
| `S-05-attention-alloc` | 6 | Pass |
| `S-06-intent-match` | 6 | Pass |
| `S-07-form-runtime` | 6 | Pass |
| `S-08-proof-checker` | 3 | Structural |
| `S-09-synth-kernel` | 8 | Pass |
| `S-10-hephaistion-seed` | 8 | Pass |
| `S-11-bridge-proto` | 6 | Pass |

**Total obligations across all proofs:** 70
**Verdicts:** 10 Pass · 1 Structural · 0 unspecified

## Helper Forms

| Metric | Value |
|---|---:|
| Helper `.form` files | 20 |
| Helpers encoded (per `STUBS.md`) | 151 |
| Helpers pending (per `STUBS.md`) | 84 |

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
| Axioms in manifest        | 10 |
| Kernel authors            | 3 |

## Axioms

| Metric | Value |
|---|---:|
| Axiom files on disk          | 10 |
| Axiom entries in manifest    | 10 |

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
| `v0.2.5-ignis0-store` | ✓ done — `Store` trait + persistent `TrieStore` HAMT backend landed; interpreter, `fixed_point`, and `Capability` repointed from concrete `SubstanceStore` to `&mut dyn Store` so either backend can be used interchangeably |
| `v0.3.0-compute` | ✓ done (`d28b466`) — landed out of order; schedule above is corrected |
| `v0.3.0-envelope+ci` | ✓ done (`e954a27`) |
| `v0.3.0-build-int` | ✓ done (`a130590`) |

**Milestone summary:** 9 done · 0 blocked · 0 other

## Drift detected

_None._ All cross-checks pass.

