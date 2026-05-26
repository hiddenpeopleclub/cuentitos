---
created: 2026-05-26
---

# Require on bool variables — implementation

Implement `req` against bool variables to satisfy the bool req compat tests.

**Prerequisite:** `require-bool-variables-compatibility-tests` done.

## Touch points

- `parser/src/parsers/requirement_parser.rs` and
  `parser/src/boolean_expression.rs` — accept `true`/`false` and the bare
  bool identifier (truthiness shortcut). Reject ordering operators on bool
  operands.
- `runtime/src/lib.rs` — comparison evaluator for the bool case.
- Type checking on `=` / `!=` operands must match.

## Verify

`cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test && ./bin/run-compat`.
