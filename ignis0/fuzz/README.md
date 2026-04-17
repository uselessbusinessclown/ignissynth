# ignis0 fuzz harness

Fuzz targets for `ignis0`'s parser surfaces. Built on
[`cargo-fuzz`](https://github.com/rust-fuzz/cargo-fuzz) (libFuzzer).

## Why

The two parsers in `ignis0` decode attacker-shaped input:

- `wire::decode_form` reads ULEB128 lengths and tag bytes from raw
  bytes. A panic here would abort the host process during ignition.
- `parser::parse_form_lines` reads the line-oriented scaffold source
  format (pre-wire, but still on the path from a `.form` file to
  executed code).

A panic on adversarial input is a security-relevant bug. See
[`../../SECURITY.md`](../../SECURITY.md).

## Targets

| Target                | Surface                                |
| --------------------- | -------------------------------------- |
| `decode_form`         | byte-exact wire codec (`wire.rs`)      |
| `parse_form_lines`    | line-oriented scaffold parser          |

Each target asserts only the panic-freedom contract. `decode_form`
additionally asserts that any successful decode round-trips through
`encode_form`.

## Running locally

```sh
# Install once
cargo install cargo-fuzz

# From ignis0/ (NOT from ignis0/fuzz/)
cd ignis0

# Smoke test (1000 inputs, ~seconds)
cargo +nightly fuzz run decode_form -- -runs=1000
cargo +nightly fuzz run parse_form_lines -- -runs=1000

# Continuous fuzzing
cargo +nightly fuzz run decode_form
```

A panic or assertion failure produces a reproducer under
`ignis0/fuzz/artifacts/<target>/`. Attach this file to a
security-labelled issue.

## CI

The fuzz harness is **not run in CI** by default — libFuzzer needs
nightly Rust and continuous-fuzzing infrastructure to be useful, and a
bounded `-runs=N` smoke test is a brittle signal. Adding a nightly
fuzz job is tracked at #14.
