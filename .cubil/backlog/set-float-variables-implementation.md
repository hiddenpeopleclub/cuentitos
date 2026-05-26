---
created: 2026-05-26
---

# Set on float variables — implementation

Implement `set` on float to satisfy the float set compat tests.

**Prerequisite:** `set-float-variables-compatibility-tests` done.

## Touch points

- `parser/src/parsers/set_parser.rs` — accept float RHS expressions and
  compound assignments when LHS is float; reject mixed int/float RHS.
- `runtime/src/lib.rs` — extend `apply_set` for `Value::Float`. Division by
  zero / overflow surface as runtime errors with wording parallel to the int
  variants.

## Verify

`cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test && ./bin/run-compat`.
