# A9 — Ignition Substrate

Derived from A0.6, A0.7, A6.5.

The IL is a specification. Specifications do not execute themselves.
Something in the physical world must implement the IL's 35-opcode
semantics before any Form can ever run — including the runtime
Form (S-07) whose body says "how to interpret IL". This axiom
names that something, defines its contract, and declares what
the habitat owes it (and does not owe it).

The naive expectation is that `form_runtime` is a Form written
in the IL it interprets, and that this is sufficient for
self-hosting. It is not sufficient, because to run S-07 you first
have to run S-07, and the recursion has no base case. The
discipline has implicitly assumed this gap would close through
synthesis; it cannot. Some interpretation of IL must exist
*before* any Form is summoned. That interpretation is what this
axiom names.

This is not the same problem as S-08's bootstrap exception. S-08's
exception is Gödelian — a proof checker cannot prove its own
soundness from within its own logic, and no amount of external
infrastructure closes that gap (only consensus among independent
reasoners does). A9's exception is Turing — the first runnable
thing has to be run by something that is already running, and
that something *can* be built and verified by ordinary means
outside the habitat.

## A9.1 — The ignition substrate exists outside the habitat

There exists, in the physical world, an implementation of the IL
that is not itself a Form. It is software written in an ordinary
language (Rust, OCaml, C, Zig, or similar), compiled to a binary
that the machine's processor executes directly. This
implementation is called the **stage-0 interpreter**, written
`ignis0`.

The habitat cannot name `ignis0` by hash because `ignis0` is not
a substance. The habitat cannot synthesise `ignis0` because
synthesis produces Forms, and `ignis0` is not a Form. The habitat
cannot verify `ignis0`'s correctness using its own proof checker
because `ignis0` is not in a language the proof checker can read.

The habitat can, however, *observe* `ignis0`'s behaviour and use
the observation to decide whether it trusts `ignis0`.

## A9.2 — `ignis0` is subject to its own discipline

`ignis0` is an ordinary software artifact. It has a repository,
a test suite, a specification (the IL.md document), and a small
set of human authors responsible for it. Its correctness with
respect to the IL.md semantics is a claim those authors make and
defend by ordinary means: tests, code review, static analysis,
fuzzing, formal verification if they choose. IgnisSynth takes no
position on which means they use.

What IgnisSynth does insist on is that `ignis0` is **not**
IgnisSynth. No act of `ignis0`'s development is recorded in the
habitat's weave. No Form in the seed mentions `ignis0` by name.
No capability is granted "to `ignis0`". The habitat knows of
`ignis0`'s existence through A9.1 alone and through the
fixed-point check at A9.3 — and through nothing else.

`ignis0`'s discipline lives in `ignis0`'s world. IgnisSynth's
discipline lives in IgnisSynth's world. The bridge between them
is the IL specification, which both worlds implement.

## A9.3 — Faithful interpretation is observable

The habitat cannot know whether `ignis0` correctly implements the
IL without running a program through it and checking the output.
A9.3 defines a specific such program, called the **ignition
fixed-point check**.

Let `F` be a small, total, well-specified Form whose output on a
canonical input is declared in IL.md (or in a sibling document
`kernel/IGNITION-BOOTSTRAP.md`). The fixed-point check is:

1. `ignis0` executes `F` directly on the canonical input. Call
   the result `r_direct`.
2. `ignis0` executes S-07 (the runtime Form, itself a sealed
   substance whose bytes are the encoded `form_runtime`) on the
   pair `(F, canonical_input)`. S-07 is a Form, so `ignis0`
   interprets S-07's IL body, which in turn interprets `F`'s IL
   body. Call the result `r_indirect`.
3. If `r_direct = r_indirect`, the check passes.

Passing the check is necessary but not sufficient for correct
interpretation: `ignis0` could be correct on `F` and wrong on
some other Form. A9.3 therefore establishes a *necessary
condition*, not a sufficient one. The fixed-point check is the
operational form of "the stage-0 interpreter is at least
self-consistent on the habitat's own runtime", and its failure
is immediate and unambiguous evidence of a stage-0 bug.

After the fixed-point check passes, the habitat declares
`ignis0` *provisionally trusted* and proceeds with ignition.
Subsequent evidence of stage-0 misbehaviour (divergent outputs
under replay, inconsistent trap semantics, violation of A3.4's
yield semantics, etc.) revokes the provisional trust and halts
the habitat, with all weave state preserved for post-mortem.

## A9.4 — Stage-0 may be replaced, but never absent

A Hephaistion synthesis act may, post-ignition, produce a new
stage-0 interpreter — for instance, by emitting a native-code
compilation of the IL that runs faster than the hand-written
`ignis0`. Such a synthesis act is permissible and is itself
recorded in the weave like any other synthesis act (because the
act is the *production* of the new interpreter, not its
execution; the production happens inside the habitat and is a
Form-emitting act).

What is *not* permissible is ever being in a state where no
stage-0 interpreter is currently running. A habitat without an
active stage-0 is a halted habitat. The transition from one
stage-0 to another is an atomic substitution performed outside
the habitat's own execution (the same way a CPU is physically
replaced while a machine is powered down), not a synthesis act
the habitat can perform on itself while running.

Concretely, replacing `ignis0` means: checkpoint the habitat
(the weave tip, the store digest, every live continuation hash),
shut it down, swap in the new stage-0, reload the checkpoint,
run the fixed-point check against the new stage-0, and resume.
The habitat's state across the substitution is byte-identical
by I7 (determinism unless declared) — which is why the seed's
entire discipline about avoiding hidden inputs and content-
addressing continuations exists.

## A9.5 — The habitat does not owe `ignis0` any authority

`ignis0` is the substrate on which the habitat runs. It is not
an inhabitant of the habitat. No capability in the cap registry
names `ignis0`. No mind has an attention tree under `ignis0`.
No intent is ever matched against `ignis0`.

If `ignis0` wants to influence the habitat — by deciding to halt
it, by refusing to execute a particular instruction, by adding
telemetry — those are interventions from outside that the
habitat records as anomalies when they become observable. They
are not inhabitant acts, not synthesis acts, not bridge acts.
They are substrate events, and the habitat is entitled to note
them in the weave as `SubstrateEvent` entries (a new entry kind
the seed loader introduces at binding time) but has no obligation
to obey them.

This is the same relationship a program has to its CPU: the CPU
can crash the program, but the program does not grant the CPU
permissions, does not model the CPU as an agent, and does not
record the CPU's internal state transitions in its own logs
except to the extent that they manifest as observable events.

## What this axiom closes

Before A9, the seed's bootstrap story had an unstated recursion:
"the kernel is a Form too, synthesised at ignition from the
axioms" (A6.5), with no answer to "what runs the synthesised
Form the first time". A9 answers that question honestly. The
first runnable thing is not a Form. It is a piece of ordinary
software whose existence is assumed, whose correctness is
provisionally trusted via a concrete fixed-point check, and
whose presence is a precondition of ignition the same way
electricity is a precondition of booting a laptop.

This is not a weakening of the discipline. It is the discipline
catching up with what was always true. The seed's proof load is
more honest with A9 than without it, because without A9 every
proof artifact was implicitly claiming to compose against an
interpreter that did not exist, and with A9 the artifacts compose
against `ignis0`'s IL-faithfulness claim, which is a bounded,
testable, falsifiable claim about a real piece of software.

## What this axiom does *not* close

A9 does not solve Gödel. S-08's inspection record and K-of-N
consensus are still required, because proof-checker soundness
cannot be tested the way interpreter faithfulness can.

A9 does not eliminate the need for v0.3.0's simulation harness
or v0.5.0's cold-start substance store. Those are still external
infrastructure the seed depends on. A9 simply names the most
fundamental piece of that infrastructure — the interpreter
itself — and gives it an axiom-level contract.

A9 does not promise `ignis0` will ever be correct. It promises
only that *if* `ignis0` is correct enough to pass the fixed-point
check, the habitat can proceed. The seed takes no further
responsibility for `ignis0`'s behaviour.

## Companion document

The operational form of A9 — what `ignis0` must accept as input,
how the fixed-point check is structured, what the canonical
input `F` actually is — lives in `kernel/IGNITION-BOOTSTRAP.md`.
