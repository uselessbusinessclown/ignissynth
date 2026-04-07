# kernel/

This directory will hold the seed Forms once they are synthesized
through the `breakdown/S-*` tasks.

It is empty by intent. IgnisSynth forbids the placement of code here
that has not gone through `synthesis/PROTOCOL.md`. Any commit that
adds a file under `kernel/forms/` must reference the synthesis chain
that produced it.

## Layout (target)

```
kernel/
  forms/
    S-01-ignite.form
    S-02-cap_registry.form
    S-03-substance_store.form
    ...
  manifest.json        # the seed manifest: hashes, signatures, axioms cited
```
