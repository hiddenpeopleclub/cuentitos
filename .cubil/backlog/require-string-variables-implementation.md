---
created: 2026-05-26
---

# Require on string variables — implementation

Implement `req` against string variables to satisfy the string req compat
tests.

**Prerequisite:** `require-string-variables-compatibility-tests` done.

## Touch points

- `parser/src/boolean_expression.rs` — accept string literals and string
  refs as operands of `=` / `!=`. Reject ordering operators on strings.
- `runtime/src/lib.rs` — string equality / inequality.
- Type checking: both sides must be string.

## Verify

`cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test && ./bin/run-compat`.
