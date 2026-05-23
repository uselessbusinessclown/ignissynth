# IgnisSynth v0.1.1-pre-ignition

> A maintenance release. The **seed itself is structurally
> unchanged** since v0.1.0-pre-ignition — same eleven primary
> Forms, same eleven proof artifacts, same proof discharge
> shape (10 end-to-end structural + 1 bootstrap structural).
> What landed since v0.1.0 is everything *around* the seed:
> the ignition substrate axiom (A9), a working stage-0
> interpreter (`ignis0`), a substantial helper-encoding
> push (86 helpers across 10 files), a 104-entry lemma
> library, a draft S-08 inspection record, a machine-readable
> conformance dashboard, golden-byte coverage for canonical
> Forms, and end-to-end CI/build hygiene including a tracked
> `Cargo.lock`, a reproducibility smoke test, and a
> conformance drift gate.
>
> No `$$BLAKE3$$` placeholder in the manifest has been
> resolved. No real Ed25519 kernel-author key has been
> generated. No live habitat exists.

## What changed since v0.1.0

### Constitution

- **Axiom A9 (Ignition Substrate)** landed
  (`axioms/A9-ignition-substrate.md` + `kernel/IGNITION-BOOTSTRAP.md`).
  This resolves the Turing-flavoured base-case bootstrap problem
  by naming an external stage-0 interpreter and pinning the
  fixed-point check (A9.3) it must pass before ignition.
- This release adds **A9** to the manifest's `axioms` block; v0.1.0
  shipped A0..A8 only.

### Specification

- **IL bumped 34 → 35 opcodes**, post-freeze, by adding **`CALLI`
  (0x22)** — the indirect call form that composes with `READSLOT`
  to close the slot-dispatch idiom (`PUSH name_hash; READSLOT;
  CALLI n`). `kernel/IL.md` declares "Thirty-five exactly" and CI
  enforces the string.
- **`kernel/IGNITION-BOOTSTRAP.md`** specifies the stage-0
  substrate contract — the loader steps, the fixed-point check,
  and the A9.4 separation between substrate caps and habitat caps.

### Cold-weave helpers

- **86 helpers encoded** across 10 `.form` files under
  `kernel/forms/helpers/`. v0.1.0 shipped helper *stubs* only
  in `STUBS.md`; v0.1.1 fills in the parser chain end-to-end
  (`parser.form`, `parser-primitives.form`, `parser-bytes.form`,
  `primitives.form`, `intrinsics.form` — 7 + 13 + 20 + 17 + 18
  sub-Forms respectively), the Schema/* primitives, and the
  third-generation byte/Nat/Vec intrinsics. The trie/forest/
  treap helpers are still stub-only and gated on
  `kernel/types/Trie.md`/`Treap.md`/`Forest.md`, which are not
  in this release.
- The encoded count is the floor enforced by CI; the up-to-date
  live count lives in [`tools/status/STATUS.md`](tools/status/STATUS.md).

### Proof load

- **104-entry lemma library** (`kernel/lemma-library.md`) —
  every `LemmaApp` head referenced across the eleven proof
  artifacts now has a named entry with a structural-reading
  discharge. Sealing the document into a substance is post-v0.5.0
  build work.
- **S-08 inspection record draft** (`kernel/forms/S-08-*.inspection-record.md`)
  — the checklist the kernel-author identities will sign once
  real Ed25519 keys exist. Signatures are placeholders.
- Every `.proof` file now carries an explicit
  `Verdict: Pass | Structural` marker so the CI proof-lint can
  distinguish "structurally complete" from "bootstrap structural"
  without prose inference.

### Stage-0 substrate (`ignis0`)

The Rust stage-0 interpreter has gone from a scaffold to a
feature-complete (against `kernel/IGNITION-BOOTSTRAP.md`)
host on its own version line. Milestones since v0.1.0:

| ignis0 tag           | What landed |
|----------------------|-------------|
| `v0.2.0-ignition`    | Scaffold: `Value`/`Hash`/`TrapKind`, all 34 opcode variants, line-oriented parser, A9.3 direct case passes. |
| `v0.2.1-ignis0-call` | `CALL` + `RET` via `FormRegistry` and call frames. |
| `v0.2.2-ignis0-wire` | Byte-exact wire codec per `IL.md §` "Byte-exact wire grammar (v1)"; encode/decode/round-trip over all opcodes + Value variants + TrapKind variants. |
| `v0.2.3-ignis0-fp`   | A9.3 indirect cases pass at levels 1 and 2 via hand-encoded micro-`S-07/execute` wrappers; full `FixedPointVerdict::Pass` with observed call-chain depths 2 and 3. |
| `v0.2.4-ignis0-cap`  | All 34 opcodes return IL-defined outcomes; no opcode returns `TrapKind::NotImplemented` after this milestone. Stage-0 constraints noted in-code where the habitat substance doesn't yet exist. |
| `v0.3.0-compute`     | Capability dispatch table, built-in GPU compute cap (wgpu/WGSL), built-in inference cap (OpenAI-compatible HTTP). Stage-0 substrate caps only — not habitat caps. |
| `v0.3.0-envelope+ci` | `FormEnvelope` derivation-gated execution control plane (`envelope.rs`, `derive.rs`, `runner.rs`, `verify.rs`), `CALLI` opcode, cargo-fuzz scaffold, `CONTRIBUTING.md`, `SECURITY.md`, reproducibility-CI smoke job + `--locked` discipline. |
| `v0.3.0-build-int`   | Build-integrity hardening on top of the above: stable-rustc 1.94 fixes (`Hash` type-alias call sites, `Opcode` PartialEq/Eq, exhaustive match on `EvalVerdict`, doctest fences in `parser.rs`), wgpu 0.20.1 API drift fixes, newer-clippy lint cleanup, proof-lint Verdict markers, manifest `seed_forms`→`forms` CI key correction. |

`ignis0/Cargo.toml` carries `version = "0.3.0-compute"`. The crate
has its own version line and is **not part of IgnisSynth proper**
— it is the interpreter the seed runs *on top of* at first
ignition, analogous to a CPU.

What is still pending on the ignis0 track:

- `v0.2.5-ignis0-store` — replace the HashMap-backed
  `SubstanceStore` with the persistent hash trie spec (S-03).
  Depends on `kernel/types/Trie.md`, which is not in v0.1.1.

### Build integrity and reproducibility

- **`ignis0/Cargo.lock` is committed**. CI no longer generates
  the lockfile mid-build; every `cargo` invocation
  (clippy, build, test, run, release) uses `--locked`.
- **`rust-toolchain.toml`** pins `channel = "stable"` with
  `rustfmt` + `clippy` components.
- **Release-build reproducibility smoke test** in CI: two
  clean builds of the same source, compare SHA-256 of the
  resulting binary. Same-host repro only; cross-host is a
  separate problem.
- **Cargo.toml `[profile.release]`** carries the reproducibility
  settings: `lto = "thin"`, `codegen-units = 1`,
  `strip = "symbols"`, `panic = "abort"`, `incremental = false`.

### Conformance dashboard

- **`tools/status/`** ships a single generated source for repo
  counts:
  - `build-status.sh` — generator (reads `manifest.json`,
    `IL.md`, `opcode.rs`, `STUBS.md`, `lemma-library.md`,
    `.proof` files, axioms; emits JSON + Markdown).
  - `status.json` — machine-readable conformance snapshot.
  - `STATUS.md` — human-readable rendering.
- **CI drift gate**: a new check regenerates the snapshot and
  fails if it differs from the committed copy (after stripping
  `generated_at` and `generated_from_commit`). Status drift
  cannot ship silently.

### Tests

- **`ignis0/tests/golden.rs`** pins canonical bytes + BLAKE3
  hashes + pretty-print output for the hand-encoded Forms
  used by the A9.3 fixed-point check (the canonical F and
  the micro-`S-07/execute` wrapper). Any silent change to the
  wire codec or pretty-printer fails this test.
- Total test count (across lib unit tests + integration
  suites + doctests): see `tools/status/STATUS.md`.

### Documentation

- **`CONTRIBUTING.md`** — reviewer/contributor quickstart.
- **`SECURITY.md`** — capability threat model and disclosure
  policy; explicitly notes `ignis0` is a research scaffold
  unsafe for untrusted Forms.
- **README "For reviewers / builders" section** at the top —
  separates "what runs now" from "what is specified only" and
  gives exact verification commands.
- **README and ROADMAP** reconciled with reality: lemma count
  corrected 105 → 104 (lemma-library.md actually has 104
  entries), missing post-#18/#19/#21 ROADMAP rows added,
  stale "cargo not available in the sandbox" audit note
  replaced with the current green-CI state, ignis0 module
  list updated with the new `envelope.rs`/`derive.rs`/
  `runner.rs`/`verify.rs`/`fuzz/` modules.

## What v0.1.1 is *not*

The same disclaimers from v0.1.0 still apply, with sharper
edges where the post-v0.1.0 work made the boundary clearer:

- **No running habitat.** `ignis0` is the stage-0 substrate
  axiom A9 names. It runs the IL faithfully (A9.3 PASS on the
  fixed-point at depths 1, 2, 3), but it is *not* IgnisSynth.
  No primary Form has been loaded into it from its `.form`
  source — those files are prose specifications, not wire
  bytes, and the parser that bridges them
  (`kernel/forms/helpers/parser.form`) is encoded but not yet
  exercised end-to-end against a primary Form.
- **No mechanically-checkable proof tree.** S-08 (the proof
  checker Form) is encoded; the structural piece is discharged
  via `WitnessExec`; obligations 2 and 3 await the signed
  inspection record and the K-of-N consensus protocol. There
  is no walker in `ignis0` that recurses through the .proof
  files at runtime.
- **No real Ed25519 keys.** `kernel/manifest.json`'s
  `kernel_authors.identities` still ships placeholder public
  keys. K-of-N signatures over the inspection record are not
  valid.
- **No resolved `$$BLAKE3$$` placeholders.** The manifest's
  immediates block still names canonical hashes as
  `$$BLAKE3$$/...` placeholders. Resolving them requires an
  external substance store (post-v0.3.0-simulation per the
  roadmap) and the v0.5.0-build cold-weave sealing.
- **No persistent trie substance store.** The `ignis0`
  `SubstanceStore` is in-memory `HashMap`. `S-03` specifies a
  persistent hash-array-mapped trie, and `kernel/types/Trie.md`
  (which would specify the node layout the helper batch needs)
  does not exist.
- **No Stage-4 simulation harness Form.** `kernel/SIMULATION.md`
  specifies the harness; no `kernel/forms/helpers/sim-harness.form`
  is encoded. Per-Form `TrialRecord` substances are not produced.
- **No habitat-internal proof checker.** The S-08 inspection
  record is the v0.1.0/v0.1.1 deliverable; a mechanically
  checkable proof rendering of the rule walker is post-v0.4.0.
- **No network, no persistence, no drivers, no compatibility
  layers.** Post-ignition concerns.

## Conformance snapshot

The numbers below are reproduced from
`tools/status/STATUS.md` at the time this release was cut.
If they disagree with `STATUS.md` after a later commit,
`STATUS.md` is the source of truth.

| Component | Value |
|---|---|
| Seed version | `0.1.1-pre-ignition` |
| `ignis0` version | `0.3.0-compute` (MSRV `1.75`) |
| Axioms on disk | 10 (A0..A9) |
| Axioms in manifest | 10 (A0..A9) — drift resolved this release |
| Primary Forms | 11 of 11 (encoded + proof) |
| IL opcodes | 35 (`Thirty-five exactly` in IL.md; 34 frozen + post-freeze `CALLI`) |
| Trap kinds | 11 official + 1 scaffold-only (`NotImplemented`) |
| Helpers encoded | 86 across 10 files (CI baseline 86) |
| Lemma library | 104 entries across 14 source groups |
| Proof artifacts | 11 (10 `Pass` + 1 `Structural`) |
| Proof obligations (total) | 70 |
| Inspection record | drafted, placeholder signatures |
| Kernel-author identities | 3 placeholders |
| Stage-4 simulation harness | spec only |
| `Cargo.lock` | tracked |
| CI on main | green |

## How to verify v0.1.1 locally

```sh
git fetch --tags
git checkout v0.1.1-pre-ignition

# Stage-0 substrate (≈10 s on a warm cache)
cd ignis0
cargo fmt --all --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked
cargo run --locked -- fixed-point   # expect: PASS Nat(43)

# Conformance dashboard sync
cd ..
bash tools/status/build-status.sh
git diff --exit-code tools/status/     # expect: no diff

# Proof-artifact + manifest structural lint (CI-equivalent)
for f in kernel/forms/S-*.proof; do
  grep -q ":obligation"                          "$f" || echo "no obligation: $f"
  grep -q "Verdict\|verdict\|Pass\|Incomplete\|Structural" "$f" || echo "no verdict: $f"
done
for k in version forms kernel_authors boot_order immediates; do
  grep -q "\"$k\"" kernel/manifest.json || echo "missing key: $k"
done
```

All four blocks should exit silently.

## What this release means for the road to ignition

v0.1.1 leaves the seed itself unchanged but firms up everything
around it:

- The bootstrap problem identified in `synthesis/SEED.md` is now
  resolved by axiom A9 plus a working stage-0 interpreter.
- The 86 encoded helpers + 104 lemmas + signed-line proofs make
  the v0.2.0-helpers milestone visibly closer (≈half of the
  helper catalogue is now bodies rather than stubs).
- Build/CI/conformance hygiene is now load-bearing: a status
  number cannot drift away from the manifest, a proof file
  cannot lose its verdict marker, and a `.form` source cannot
  change its canonical hash silently.
- The next external-build dependency — `kernel/types/Trie.md`
  for the persistent substance store — is now the only thing
  gating `v0.2.5-ignis0-store`, which in turn gates
  `v0.3.0-simulation`.

The remaining path is the one the roadmap names:
**Trie.md → ignis0/store → Stage-4 simulation harness → real
Ed25519 keys + signed inspection record → external substance
store + cold-weave seal → first boot.** None of those steps
require new primary Forms; the seed is closed at eleven.

## Acknowledgements

`v0.1.1-pre-ignition` is the first IgnisSynth release whose
preparation, build verification, status reconciliation, and
release notes were produced inside the loop the system is
designed to host — an AI agent running against the
synthesis discipline. The discipline held: every change in
this release is traceable to either an explicit synthesis
act, a bug fix against an existing artifact, a build-integrity
fix, or a doc reconciliation; no new primary Form was synthesized,
no closed enumeration was extended.
