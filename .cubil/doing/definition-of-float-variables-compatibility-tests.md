---
created: 2026-05-26
---

# Definition of float variables — compatibility tests

Compat tests for declaring float variables. TDD.

## Feature summary

```cuentitos
--- variables
float health
float starting_health = 10.5
float bonus = starting_health * 2.0
---
```

- `float <name>` — defaults to `0.0`.
- `float <name> = <expr>`.
- Literal syntax: `1.5` form only — **no** `1e3`, no bare `.5`, no trailing
  `1.`. Negative literals via unary minus.
- Default expressions: `+ - * /`, parentheses, float literals, references to
  earlier float variables. Same eager / constant-folded semantics as int.
- Division semantics for floats: standard IEEE behaviour (no truncation).
  Division by zero in a default is a parse-time error.

## What to cover

- `feature/` — declaration with/without default, arithmetic in default,
  references to earlier floats, `?` output (decide and document the format
  in the tests — e.g. `name: 10.5`).
- `errors/` — invalid literal (`1e3`, `.5`, `1.`), default division by zero,
  default overflow (extremely large product), forward reference, duplicate
  name, reserved keyword.
- `edge-cases/` — `-0.0`, very small / very large finite values, mixing with
  int and bool in the same block.

## Reference

Mirror `compatibility-tests/variables-integer/`. Counterpart (done):
`definition-of-integer-variables-compatibility-tests`.
