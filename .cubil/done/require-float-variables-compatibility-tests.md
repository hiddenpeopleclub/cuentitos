---
created: 2026-05-26
---

# Require on float variables — compatibility tests

Compat tests for `req` against float variables. TDD.

## Feature summary

All six comparison operators (`= != < <= > >=`) work on floats. `=` and `!=`
ARE allowed — the author is responsible for any precision pitfalls.

```cuentitos
--- variables
float health = 10.0
---
Alive.
  req health > 0.0
Full health.
  req health = 10.0
```

- Each side of a comparison may be a float arithmetic expression.
- Combines with `and`/`or`/`not`.

## What to cover

- `feature/` — all six operators, arithmetic on each side, multiple sibling
  `req`s, combinations with `and`/`or`/`not`, req after a set.
- `errors/` — float vs non-float comparison (type mismatch), division by
  zero in req expression (runtime), undeclared variable.
- `edge-cases/` — `-0.0 = 0.0` behaviour (decide and document), comparing
  to a default-init `float x` (value `0.0`).

## Reference

`require-integer-variables-compatibility-tests` (done).
