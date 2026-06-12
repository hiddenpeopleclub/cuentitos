---
created: 2026-05-26
---

# Require on enum variables — implementation

Implement `req` against enum variables to satisfy the enum req compat tests.

**Prerequisite:** `require-enum-variables-compatibility-tests` done.

## Touch points

- `parser/src/boolean_expression.rs` — accept enum value names and same-enum
  variable refs as operands of `=` / `!=`. Reject ordering operators on enums.
- Parse-time validation that each value name belongs to the variable's enum.
- `runtime/src/lib.rs` — comparison evaluator for enums; reading an unset
  enum surfaces a runtime error with wording parallel to existing runtime
  errors.

## Verify

`cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test && ./bin/run-compat`.
