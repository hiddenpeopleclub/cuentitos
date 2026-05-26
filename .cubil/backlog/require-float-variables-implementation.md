---
created: 2026-05-26
---

# Require on float variables — implementation

Implement `req` against float variables to satisfy the float req compat tests.

**Prerequisite:** `require-float-variables-compatibility-tests` done.

## Touch points

- `parser/src/boolean_expression.rs` — accept float arithmetic expressions
  on each side of a comparison.
- `runtime/src/lib.rs` — comparison evaluator for the float case (use
  standard `f64` comparisons; lock in `-0.0 = 0.0` semantics per the tests).
- Type checking: both sides of a comparison must be the same numeric type
  (no implicit int↔float coercion in v1).

## Verify

`cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test && ./bin/run-compat`.
