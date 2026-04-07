# kernel/

This directory will hold the seed Forms once they are synthesized
through the `breakdown/S-*` tasks.

It is empty by intent. IgnisSynth forbids the placement of code here
that has not gone through `synthesis/PROTOCOL.md`. Any commit that
adds a file under `kernel/forms/` must reference the synthesis chain
that produced it.

## Layout

```
kernel/
  IL.md                            # the Form intermediate language specification
  forms/
    S-01-ignite.form               # ✓
    S-02-cap-registry.form         # ✓
    S-03-substance-store.form      # ✓
    S-04-weave-log.form            # ✓
    S-05-attention-alloc.form      # ✓
    S-06-intent-match.form         # ✓
    S-07-form-runtime.form         # ✓
    S-08-proof-checker.form        # ✓
    S-09-synth-kernel.form         # ✓
    S-10-hephaistion-seed.form     # ✓
    S-11-bridge-proto.form         # ✓
  manifest.json                    # (next) the seed manifest: hashes, signatures
```

## Status

`IL.md` defines the 30-opcode Form intermediate language. All eleven
seed Forms are encoded against it. Each Form references the helper
Forms it depends on by slot name (`READSLOT` + `CALL`); the helpers
themselves (canonicaliser, trie ops, why-traversal, lemma library,
etc.) are the next layer of encoding work, alongside the proof
artifacts named in each breakdown's Proof section and the seed
manifest binding all the hashes together.
