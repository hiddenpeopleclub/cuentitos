---
created: 2026-05-26
---

# Definition of bool variables — compatibility tests

Write the compat tests for **declaring** bool variables. Sibling task:
`definition-of-bool-variables-implementation`. TDD: tests first.

## Feature summary

Bool variables live in the existing `--- variables` block.

```cuentitos
--- variables
bool a_default_bool
bool starts_true = true
bool starts_false = false
bool mirror = starts_true
---
```

- `bool <name>` — defaults to `false`.
- `bool <name> = true|false`.
- Default RHS allowed: literal `true`/`false`, or a reference to a
  previously-declared bool variable. **No** inline `and`/`or`/`not` in defaults
  — those live in `req`.
- `<name>` rules same as int.

## What to cover

- `feature/` — declaration with/without default, ref to earlier bool, mixing
  with int, `?` output (`name: true` / `name: false`).
- `errors/` — default isn't `true`/`false`, default references a non-bool,
  default uses logical operators, forward reference, duplicate name across
  types, `bool and = true` (reserved word).
- `edge-cases/` — empty `--- variables` still valid, bool/int interleaved.

## Reference

Mirror `compatibility-tests/variables-integer/`. Counterpart task (done):
`definition-of-integer-variables-compatibility-tests`.
