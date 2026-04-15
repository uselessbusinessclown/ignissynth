# Security Policy

This document covers the security posture of the `ignissynth`
research repository. It is **not** a production security policy: nothing
in this repo is intended to run untrusted Forms or untrusted substances
in a hostile environment.

## Scope

Two artefacts live in this repo:

1. **The IgnisSynth specification** (`kernel/`, `axioms/`, `synthesis/`,
   `breakdown/`, `docs/`) — Markdown, `.form` files, `.proof` files,
   and `manifest.yaml`. These are *specifications*, not running code.
   Security-relevant bugs here are inconsistencies, ambiguities, or
   contract violations between sections.

2. **The `ignis0` stage-0 substrate** (`ignis0/`) — a Rust crate that
   implements the IL interpreter described in `kernel/IL.md`. This is
   *executable* code and has a real attack surface (parser, wire codec,
   capability dispatch).

Per axiom A9.2, `ignis0` is an *external* substrate that IgnisSynth
depends on. It lives in this repo only during the scaffold phase. The
security posture below applies to both, but the threat model is
substantially different.

## Non-goals for the stage-0 scaffold

`ignis0` v0.x is a research scaffold. The following are **explicitly
out of scope** and should not be assumed to hold:

- **No sandboxing.** `ignis0` runs Forms in-process. A malicious Form
  with the right capabilities can read the file system, make network
  calls (via the inference cap), and dispatch GPU compute. Do not run
  Forms you did not author.
- **No proof checker.** Step 4 of the ignition sequence
  (`kernel/IGNITION-BOOTSTRAP.md` § Step 4) is not implemented. Sealed
  substances are content-addressed but not *verified* against an
  obligation. The `S-08` inspection record check is a stub.
- **No signed substances.** There is no signature step on `S-03.seal`
  and no signature check on `S-03.read`. Substance hashes only attest
  to byte content, not to provenance.
- **No persistent store.** `SubstanceStore` is an in-memory `HashMap`.
  No on-disk integrity to protect.
- **No multi-tenant isolation.** A single `Interpreter` shares one
  `CapabilityRegistry` and one `SubstanceStore` across all call frames.
  A capability invocation can mutate state that a sibling frame later
  reads.

If you need any of these properties, do not use `ignis0` v0.x.

## Capability threat model

The `INVOKE` opcode is the only IL surface that crosses the
substrate/Form boundary. All non-IL effects flow through it:

| Capability | Descriptor | What it can do | Trust level |
| --- | --- | --- | --- |
| `Synthesis/infer/v1` | `ignis0/capability/Synthesis/infer/v1` | HTTP POST to the inference server URL configured by `IGNIS0_INFER_URL` | **Trusted to the URL** — defaults to `http://localhost:11434`, but env-overridable |
| `Compute/gpu/v1` | `ignis0/capability/Compute/gpu/v1` | Compile and dispatch arbitrary WGSL via wgpu (Vulkan/Metal/DX12/WebGPU) | **Trusted to the GPU driver** — shader bugs can hang or crash the driver |

Both built-ins are gated behind cargo features (`infer`, `gpu`). With
neither feature compiled in, `INVOKE` traps `ENotHeld` on every cap;
that is the most restrictive useful posture for the scaffold.

### Trust assumptions

- **Cap registration is privileged.** Anyone who can construct the
  `CapabilityRegistry` (i.e. the host process embedding `ignis0`) can
  bind any `Hash` to any backend. There is no run-time check that a
  registered cap matches its canonical descriptor; the host is trusted
  to register honestly.
- **`from_env()` reads environment variables.** `CapabilityRegistry::from_env()`
  and `InferenceConfig::from_env()` read `IGNIS0_INFER_*` env vars. A
  process that inherits a hostile environment will dispatch to a
  hostile inference server. Not for production use.
- **CapId stability is a hard interface.** The BLAKE3 hashes of the
  descriptor strings in `ignis0/src/capability.rs` are part of the
  ABI: every Form that pushes them at an `INVOKE` site depends on
  them. Changing a descriptor string is a breaking change.

### What `INVOKE` does NOT do

- It does not check that the caller "holds" the capability in any
  meaningful sense beyond `cap_view.contains(c)`. There is no
  delegation, no expiration, no audit trail.
- It does not rate-limit. A Form in a tight loop can saturate the
  inference server.
- It does not isolate one Form's `INVOKE` from another's. A capability
  backend that mutates global state (e.g. a stateful inference session)
  will leak between Forms.

These gaps are tracked under the **Substrate Hardening** milestone.

## Wire codec attack surface

`ignis0/src/wire.rs` decodes byte-exact Form representations into
`Opcode` and `Value` trees. It must not panic on adversarial input — a
panic in `decode_form` would abort the host process. The current test
suite covers all 35 opcodes and 11 trap kinds round-trip; there is **no
fuzzing**. Adding a fuzz harness is tracked at #14.

If you find a panic on adversarial input, report it as a security issue
(see "Disclosure" below) rather than a normal bug.

## Disclosure

This is a research repository with no SLA. That said:

- **Prefer GitHub Security Advisories** for anything that could be a
  real exploit on a `ignis0` host: parser panics, capability escapes,
  hash collisions, signature forgeries (when signatures land).
- **Open a public issue** for spec-level inconsistencies, contract
  bugs, or anything that is clearly a bug but not exploitable.
- Do not include zero-day exploits in public commits or PRs.

When reporting, please include:

- The commit SHA the report applies to
- A reproduction (Form bytes, IL trace, or the `.form` source)
- Expected vs actual behaviour (which trap fires, or which trap
  *should* fire and does not)
- For substrate bugs: the cargo features used (`infer`, `gpu`, or none)

We will respond on a best-effort basis.

## Boundary summary

```
┌────────────────────────────────────────────────────────────┐
│ Host process (your code)                                   │
│   ┌─────────────────────────────────────────────────────┐  │
│   │ ignis0::Interpreter                                 │  │
│   │   ┌────────────────────┐   ┌──────────────────────┐ │  │
│   │   │ IL execution       │   │ CapabilityRegistry   │ │  │
│   │   │ (35 opcodes, pure) │──▶│ (THIS IS THE BOUNDARY) │ │
│   │   └────────────────────┘   └──────────┬───────────┘ │  │
│   └────────────────────────────────────────┼─────────────┘  │
│                                            ▼                │
│                              ┌─────────────────────────┐    │
│                              │ Inference HTTP / wgpu   │    │
│                              └─────────────────────────┘    │
└────────────────────────────────────────────────────────────┘
```

Everything inside `IL execution` is a pure interpreter over an
in-memory store. Everything outside the `CapabilityRegistry` boundary
is the host's responsibility to constrain.
