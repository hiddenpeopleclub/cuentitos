---
created: 2026-05-26
---

# Set on string variables — implementation

Implement `set` on string to satisfy the string set compat tests.

**Prerequisite:** `set-string-variables-compatibility-tests` done.

## Touch points

- `parser/src/parsers/set_parser.rs` — accept string literals and string
  refs when LHS is string; reject `+`, compound assignment, type mismatch.
- `runtime/src/lib.rs` — extend `apply_set` for `Value::String`.

## Verify

`cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test && ./bin/run-compat`.
