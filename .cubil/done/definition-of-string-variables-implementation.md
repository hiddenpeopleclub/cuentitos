---
created: 2026-05-26
---

# Definition of string variables — implementation

Implement string declarations to satisfy `compatibility-tests/variables-string/`.

**Prerequisite:** `definition-of-string-variables-compatibility-tests` done.

## Touch points

- `common/src/value.rs` — add `Value::String(String)`.
- `parser/src/parsers/variables_parser.rs` — recognise `string`; parse
  double-quoted literals with `\" \n \\` escapes; reject other escapes and
  multi-line literals.
- `?` pretty-printer — canonical format locked by the compat tests.

## Decisions baked in

- Default is `""` when no `=`.
- Only `\"`, `\n`, `\\` escapes supported. No multi-line literals. No
  interpolation. No concatenation (separate task).

## Verify

`cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test && ./bin/run-compat`.
