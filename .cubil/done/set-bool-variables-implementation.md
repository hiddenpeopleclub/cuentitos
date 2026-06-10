---
created: 2026-05-26
---

# Set on bool variables — implementation

Implement `set` on bool to satisfy the bool set compat tests.

**Prerequisite:** `set-bool-variables-compatibility-tests` done.

## Touch points

- `parser/src/parsers/set_parser.rs` — accept `true`/`false` and bool refs
  when LHS is bool; reject arithmetic RHS and compound assignment for bool.
- `runtime/src/lib.rs` — extend `apply_set` for `Value::Bool`.
- Type-mismatch error wording parallel to existing int diagnostics.

## Verify

`cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test && ./bin/run-compat`.
