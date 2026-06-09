---
created: 2026-05-26
---

# Definition of enum variables — compatibility tests

Compat tests for declaring enum variables. TDD.

## Feature summary

Enum variables live inside the existing `--- variables` block. The variable
and its allowed values are declared in one line.

```cuentitos
--- variables
enum mood = happy, sad, angry
enum weather = sunny, rainy
---
```

- Syntax: `enum <name> = <value1>, <value2>, ...`. Value list is REQUIRED
  (no shorthand without values).
- **No implicit starting value.** Enum variables are *unset* until a first
  `set` runs. Reading an unset enum is a runtime error.
- `<value>` rules: same identifier rules as variable names; no duplicates
  within the same enum; values are local to their enum (two enums may share
  a value name, and references are always qualified by the variable).

## What to cover

Scope is **declaration only**. Every test here must be greenable by the
declaration implementation alone — no test may depend on `set` (that belongs to
`set-enum-variables-compatibility-tests`). `?` is treated as a debug inspector
that reports `<unset>` and never triggers the "reading an unset enum is a
runtime error" rule (that rule only applies to `req`/expression reads).

- `feature/` — single enum declaration, multiple enums in one block, mixing
  with other types, `?` output for unset (locked as `mood: <unset>`).
  (`?` after a `set` moved to `set-enum-variables-compatibility-tests`.)
- `errors/` — empty value list (`enum mood =`), duplicate value within an
  enum, value-list value matches a reserved word, value that is not a valid
  identifier, declaration without `=`, same variable name declared twice.
- `edge-cases/` — value list with whitespace variants (asserted at declaration
  time: messy whitespace still parses to `<unset>`), two enums sharing a value
  name (legal — the duplicate-value check is per-enum, not global). The deeper
  reference-scoping proof that needs `set` lives in the set/req suites.

## Reference

Mirror `compatibility-tests/variables-integer/` structure. New layout:
`compatibility-tests/variables-enum/`.
