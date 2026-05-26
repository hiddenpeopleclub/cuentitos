---
created: 2026-05-26
---

# Definition of bool variables — implementation

Implement bool declarations to satisfy `compatibility-tests/variables-bool/`.

**Prerequisite:** `definition-of-bool-variables-compatibility-tests` done.

## Touch points

- `common/src/value.rs` (or wherever `Value` lives) — add `Value::Bool(bool)`.
- `parser/src/parsers/variables_parser.rs` — recognise `bool`; parse
  `true`/`false` literals; allow bool variable refs in defaults; reject inline
  logical operators in defaults.
- `?` pretty-printer — `name: true` / `name: false`.
- Error wording — match the style of the int defaults messages (parallel
  shape, only the keyword differs).

## Decisions baked in

- Default is `false` when no `=`.
- Default RHS: literal `true`/`false` or earlier bool variable. No `and`/`or`/`not`.

## Verify

`cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test && ./bin/run-compat`.
