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
  IL.md                # the Form intermediate language specification
  forms/
    S-01-ignite.form   # ✓ encoded
    S-02-cap_registry.form
    S-03-substance_store.form
    ...
  manifest.json        # the seed manifest: hashes, signatures, axioms cited
```

## Status

`IL.md` defines the 30-opcode Form intermediate language that every
seed Form is written in. `forms/S-01-ignite.form` is the first
worked encoding. The remaining ten Forms are written against the
same IL and committed as their encodings are produced.
