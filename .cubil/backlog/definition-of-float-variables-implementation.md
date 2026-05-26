---
created: 2026-05-26
---

# Definition of float variables — implementation

Implement float declarations to satisfy `compatibility-tests/variables-float/`.

**Prerequisite:** `definition-of-float-variables-compatibility-tests` done.

## Touch points

- `common/src/value.rs` — add `Value::Float(f64)`.
- `parser/src/parsers/variables_parser.rs` — recognise `float`; parse `1.5`
  form literals; reject `1e3`, bare `.5`, trailing `1.`.
- Share the arithmetic body via the existing `ArithmeticSource` (extended for
  float, or a parallel `FloatArithmeticSource` — pick the cleaner shape and
  document the choice).
- `?` pretty-printer — pick a canonical format and lock it in the compat
  tests.

## Decisions baked in

- Default is `0.0` when no `=`.
- Literal grammar is `1.5` only.
- Standard float division (no truncation). Division by zero in default →
  parse-time error.

## Verify

`cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test && ./bin/run-compat`.
