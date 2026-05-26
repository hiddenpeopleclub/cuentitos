---
created: 2026-05-26
---

# Set on float variables — compatibility tests

Compat tests for `set` on float variables. TDD.

## Feature summary

Mirrors the int `set` story: simple assignment, arithmetic RHS, compound
assignments.

```cuentitos
--- variables
float health = 10.0
---
set health = 5.5
set health += 2.5
set health *= 0.5
```

- Compound forms: `+= -= *= /=`.
- Mixed float/int arithmetic on RHS is a type error (no implicit coercion in
  v1 — keep the type system strict).

## What to cover

- `feature/` — simple set, arithmetic RHS, all four compound forms, set
  using self-reference (`set health = health - 1.0`).
- `errors/` — RHS division by zero (runtime), RHS overflow (runtime), RHS
  references undeclared variable, RHS uses int literal where float expected,
  comparison operator on RHS.
- `edge-cases/` — set inside section / indented block, multiple sets in
  sequence reflected in `?`.

## Reference

`set-integer-variables-compatibility-tests` (done).
