---
created: 2026-05-26
---

# Set on enum variables — compatibility tests

Compat tests for `set` on enum variables. TDD.

## Feature summary

```cuentitos
--- variables
enum mood = happy, sad, angry
---
set mood = happy
You feel okay.
set mood = sad
```

- RHS: a value name from the variable's declared list, OR a reference to
  another enum variable (must be the SAME enum — i.e. same variable, or this
  is a type error in v1).
- Setting to a value not in the list is a **parse-time** error.
- No compound assignment.

## What to cover

- `feature/` — set to each declared value, multiple sets in sequence, `?`
  reflects the change (and `?` before first set shows unset). Includes the
  `query-after-set` test moved here from the declaration suite, plus setting to
  a value that was declared with surrounding whitespace (e.g. `enum mood =
  happy ,sad,  angry` then `set mood = angry`).
- `errors/` — set to value not in list (parse-time), set to another enum
  variable of a different enum (type mismatch), set to a string/int/bool
  literal, set to an undeclared name.
- `edge-cases/` — set inside section / indented block, set before any read
  flips state from unset to a value.

## Reference

`set-integer-variables-compatibility-tests` (done).
