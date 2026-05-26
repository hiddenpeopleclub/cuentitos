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

- `feature/` — single enum declaration, multiple enums in one block, mixing
  with other types, `?` output for unset (`mood: <unset>` or similar — decide
  and document), `?` after a `set`.
- `errors/` — empty value list (`enum mood =`), duplicate value within an
  enum, value-list value matches a reserved word, declaration without `=`,
  same variable name declared twice.
- `edge-cases/` — value list with whitespace variants, two enums sharing a
  value name (legal since each value reference is scoped to its variable).

## Reference

Mirror `compatibility-tests/variables-integer/` structure. New layout:
`compatibility-tests/variables-enum/`.
