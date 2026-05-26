---
created: 2026-05-26
---

# Set on enum variables — implementation

Implement `set` on enum to satisfy the enum set compat tests.

**Prerequisite:** `set-enum-variables-compatibility-tests` done.

## Touch points

- `parser/src/parsers/set_parser.rs` — when LHS is enum, RHS must be a value
  in that enum's list (validated at parse time) or another enum variable of
  the same enum. Reject everything else.
- `runtime/src/lib.rs` — extend `apply_set` for the enum case; clear the
  unset flag.
- Error wording for "value not in this enum's list" parallel to existing
  type-mismatch wording.

## Verify

`cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test && ./bin/run-compat`.
