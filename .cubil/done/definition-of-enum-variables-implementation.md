---
created: 2026-05-26
---

# Definition of enum variables — implementation

Implement enum declarations to satisfy `compatibility-tests/variables-enum/`.

**Prerequisite:** `definition-of-enum-variables-compatibility-tests` done.

## Touch points

- `common/src/value.rs` — add `Value::Enum { variant: EnumVariantId }` or
  similar. Likely also `Value::EnumUnset { enum_id }` to represent the unset
  state at runtime. Pick the shape that makes the runtime error cleanest.
- `parser/src/parsers/variables_parser.rs` — recognise `enum`; parse the
  comma-separated value list; trim whitespace around `=` and commas; check for
  duplicates; check for reserved-word values; check each value is a valid
  identifier (same rule as variable names).
- `?` pretty-printer — lock the unset format as `mood: <unset>`.

## Decisions baked in

- Value list always required.
- No implicit starting value; enum is *unset* until first `set`.
- Reading an unset enum is a RUNTIME error — but `?` is a debug inspector and
  is EXEMPT: it reports `<unset>` and must not raise. The runtime-read-error
  path is exercised only by the `req` suite.
- Value names scoped to their variable (two enums may share a value name); the
  duplicate-value check is per-enum, not global.

## Verify

`cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test && ./bin/run-compat`.
