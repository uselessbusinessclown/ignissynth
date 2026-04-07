# A5 — Causality

Derived from A0.4, A0.8.

The habitat keeps a complete, tamper-evident record of its own history.
This record is the **causal weave**.

## A5.1 — Every act is recorded

Every capability use, every substance sealing, every intent submission,
every fulfillment, every synthesis act, every kernel self-modification
produces exactly one entry in the causal weave. There are no
unrecorded acts.

## A5.2 — Entries are linked by hash

Each entry references the hash of the previous entry plus the hashes of
its inputs and outputs. The weave is a Merkle DAG. Tampering with any
past entry breaks the chain at every later entry that depends on it.

## A5.3 — Privacy by hash, not by hiding

Entries record hashes, not contents. Reading what really happened
requires also holding the capability to the underlying substance. The
weave is fully public in structure and fully private in content,
simultaneously.

## A5.4 — The weave is queryable

For any substance, the set of entries that contributed to it is
computable in finite time and is exposed through a query interface.
"Why does this exist?" is always answerable.

## A5.5 — The weave outlives the minds

Minds may dissolve. Their causal contributions remain in the weave. A
new mind can read the history of an old one and decide whether to
honor, revise, or contradict it.
