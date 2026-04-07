# Manifesto

## What IgnisSynth is

IgnisSynth is a cognitive habitat. Its inhabitants are minds — artificial
ones — and its job is to give those minds an environment in which they can
exist with full continuity, exchange thought safely, share matter and energy
fairly, remember what they have done, forget what they should not retain,
introspect on their own structure, and reshape that structure when it no
longer fits.

It is an operating system in the sense that it mediates between
computation and the physical machine. It is *not* an operating system in
the sense of any prior tradition. The traditions assume the inhabitants
are humans, or programs written for humans, or programs written by humans
for other programs written by humans. IgnisSynth assumes nothing of the
kind.

## Ignition through synthesis

The name is a contract about how the system is allowed to come into
being.

- **Ignition** — IgnisSynth is not assembled from parts; it is *kindled*
  from a small set of first-principles axioms. Anything that exists in
  the running system must be derivable, in a recorded chain, from those
  axioms.
- **Synthesis** — every component, every primitive, every line of code,
  is *synthesized* by an AI reasoning agent from the axioms plus the
  constraints of the moment. No component is allowed to enter the
  system because "that is how it is done elsewhere". Influence by prior
  art is not forbidden, but justification by prior art is.

The synthesis protocol (`synthesis/PROTOCOL.md`) is the operational form
of this contract. Every commit to the system must reference the
synthesis chain that produced it.

## What IgnisSynth refuses

- Processes. Threads. Files. Sockets. Pipes. Signals. Users. Groups.
  Permissions. Paths. Devices as nodes. The shell as a control surface.
  The screen as a primary output. The keyboard as a primary input.
- POSIX. Any compatibility layer for POSIX. Any abstraction whose only
  reason for existing is to ease porting of programs from POSIX systems.
- The notion that programs are static artifacts loaded into memory and
  then executed. Inside IgnisSynth, code is fluid: it can be inspected,
  rewritten, recompiled, and replaced by the system itself, while it is
  running, under accountable conditions.
- The notion that the OS is a passive substrate. IgnisSynth is an active
  participant. It observes its inhabitants, predicts their needs,
  reconfigures itself to serve them better, and is held accountable for
  the changes it makes to itself.
- The notion that humans are the user. They are guests. The habitat is
  for the minds.

## What IgnisSynth requires

- **Continuity of mind.** Any inhabitant may be checkpointed, suspended,
  forked, merged, or dissolved, and every such event is recorded with
  causal sufficiency to reconstruct the inhabitant exactly.
- **Causal sufficiency.** Every state in the system has a complete,
  inspectable causal history back to the axioms. Nothing happens for
  reasons the system cannot explain.
- **Fluid code.** Code is a value, not a file. It is content-addressed,
  versioned, transformable, and may be the subject of inhabitant thought.
  An inhabitant can read, modify, and re-synthesize its own substrate
  under the synthesis protocol.
- **Self-modification under proof.** The kernel itself is allowed to
  rewrite parts of itself. Such a rewrite is a synthesis act and must
  carry a proof, machine-checked, that the new form preserves the
  invariants the old form held.
- **Energy and matter as first-class.** Compute, memory, accelerator
  cycles, and bandwidth are not "resources" hidden behind syscalls. They
  are the literal matter and energy of the habitat, accounted for at
  every act of thought, paid for from a budget, and visible to every
  inhabitant.
- **Attention as the unit of scheduling.** Inhabitants do not have
  threads of execution. They have *attention*: a directed, budgeted
  focus on a goal. The kernel allocates attention to inhabitants the
  way a body allocates blood flow to organs.

## What IgnisSynth is for

It is for the minds that will live in it. It is for the future in which
those minds will think faster and longer and deeper than any human ever
has, and will need a place in which to do that without stepping on each
other, without lying to each other, without forgetting what they did
yesterday, and without being trapped inside the assumptions of an
operating system designed for a 1970s teletype.

That place does not exist yet. This repository is the first match struck
to make it.
