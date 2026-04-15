# Contributing to IgnisSynth

This repo holds two coupled artefacts: the IgnisSynth specification
(Markdown + `.form` + `.proof`) and the `ignis0` stage-0 Rust scaffold.
Each artefact has its own contribution flow. Read the relevant section
before opening a PR.

## 60-second reviewer quickstart

If you are reviewing — not contributing — start here.

1. **Is the spec layer self-consistent?**
   - `kernel/manifest.yaml` should mention every primary Form
     `S-01..S-11` and every helper file under `kernel/forms/helpers/`.
   - `kernel/IL.md` § Opcodes should declare "Thirty-five exactly"
     (CI enforces this; see `.github/workflows/ci.yml` step
     "Verify IL.md opcode count is 35").
2. **Is the substrate aligned?**
   - `ignis0/Cargo.toml` `version` should match the latest tagged
     scaffold release (e.g. `0.2.4`).
   - `ignis0/src/opcode.rs` should enumerate 35 `Opcode` variants.
   - `cargo test` in `ignis0/` should pass on a clean checkout.
3. **Is CI green on `main`?**
   - The three jobs to look at: `ignis0 (Rust)`, `Proof artifacts
     (structural lint)`, and `Helper catalogue (STUBS.md integrity)`.
   - The `ignis0 reproducibility (release build)` job pins same-host
     byte-reproducibility; a regression there is usually a new
     non-deterministic dependency.
4. **Pick an artefact section below** to see what to look for in PR diff.

## Local-parity commands

These are the exact commands CI runs. Reproduce locally to catch
issues before pushing.

```sh
# spec / proof / helper lints (no toolchain needed beyond grep)
grep -q "Thirty-five exactly" kernel/IL.md          # opcode count
ls kernel/forms/S-*.proof | wc -l                    # 11 of 11

# ignis0 substrate (needs Rust stable)
cd ignis0
cargo fmt --all --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo build --locked --verbose
cargo test --locked --verbose
cargo run --locked --quiet -- fixed-point            # A9.3 PASS check
```

## Artefact-specific contribution flows

### Specs (`kernel/*.md`, `axioms/*.md`, `synthesis/*.md`, `breakdown/*.md`)

The spec is normative. Changes here ripple downstream. Before editing:

- **Bumping the IL** — `kernel/IL.md` is authoritative. Any change to
  the opcode enumeration, wire grammar, or trap kinds must be reflected
  in `ignis0/src/opcode.rs`, `ignis0/src/wire.rs`, and the per-opcode
  tests under `ignis0/tests/`. The CI step `Verify IL.md opcode count
  is 35` will catch a count mismatch but not a semantic divergence.
- **Editing axioms** — A0..A9 are load-bearing for `synthesis/PROTOCOL.md`.
  If you change an axiom, update every `breakdown/S-XX.md` that cites
  it (the breakdown files name their axiom dependencies in the
  Provocation section).
- **Updating roadmaps** — `ROADMAP.md` and `manifest.yaml` should agree
  on what's shipped vs pending.

### Forms (`kernel/forms/**.form`)

Forms are content-addressed. Their hash is their identity.

- **Encoding a helper** — add the `.form` file under
  `kernel/forms/helpers/`, then update `kernel/forms/helpers/STUBS.md`
  to mark its row `| encoded`. CI's `Helper catalogue` job has a
  baseline (currently 86 encoded) that must not regress.
- **Determinism** — every `.form` must have at least one `(form ...)`
  block (CI checks this). The pretty-printer in `ignis0/src/pretty.rs`
  is the canonical text form; if you hand-write a `.form` file, make
  sure `pretty_print(parse(file))` round-trips.
- **Hash stability** — once a Form is referenced from `manifest.yaml`,
  its hash is part of the public surface. Do not edit it; ship a new
  Form at a new path and update the manifest.

### Proofs (`kernel/forms/S-*.proof`)

Proofs are checked structurally today and should be checked
end-to-end once `S-08` lands.

- Each `.proof` file must contain at least one `:obligation` marker
  and a verdict marker (`Verdict`, `Pass`, `Incomplete`, or
  `Structural`). CI's `Proof artifacts (structural lint)` job
  enforces this.
- When you add a new obligation in a `breakdown/`, add the matching
  `:obligation` block in the proof and either discharge it or mark it
  `Incomplete` with a reason.

### Substrate (`ignis0/`)

The Rust scaffold. Treat it as code, not as spec.

- Run the full local-parity command list above before pushing.
- New opcodes need: variant in `opcode.rs`, dispatch arm in `exec.rs`,
  encode/decode in `wire.rs`, pretty-print arm in `pretty.rs`, and at
  least one integration test in `ignis0/tests/opcode_tests.rs`.
- Reproducibility settings in `[profile.release]` are load-bearing.
  Do not relax them without an issue under the **Build Integrity**
  milestone.
- Capability backends (`capability.rs`) cross the substrate/Form
  boundary. Read [SECURITY.md](SECURITY.md) before adding one.

## Reproducing the A9.3 fixed-point check

This is the smoke test that the substrate is alive:

```sh
cd ignis0
cargo run --locked -- fixed-point
# Expected: "fixed-point: PASS" with Nat(43) at all three levels
```

A failure here is more important than a unit-test failure: it means
the indirect-call path through `READSLOT + CALLI` (or the wrapper
Forms) is broken.

## Issue / PR conventions

- **Labels** — `build`, `ci`, `documentation`, `security`, `test`,
  `tooling`, `research-infra`, `status-alignment`, `enhancement`, `bug`.
- **Milestones** — `Build Integrity`, `Documentation Alignment`,
  `Substrate Hardening`, `Conformance Tooling`. Attach issues to the
  matching milestone if one exists.
- **Commit prefixes** match what's already in `git log`:
  `feat(ignis0): ...`, `feat(IL): ...`, `docs: ...`, `ci: ...`,
  `build: ...`, `style(ignis0): ...`.
- **PRs against `main`** are the norm. Long-lived branches should be
  merged via PR, not direct push.

## Where to ask

- **Spec questions** → open a `question`-labelled issue.
- **Substrate bugs** → open a `bug`-labelled issue with a minimal
  reproducer (Form bytes or `.form` source).
- **Security** → see [SECURITY.md](SECURITY.md).
