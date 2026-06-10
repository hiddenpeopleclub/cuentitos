---
created: 2026-05-26
---

# Set on bool variables — compatibility tests

Compat tests for `set` on bool variables. TDD.

## Feature summary

```cuentitos
--- variables
bool door_open = false
---
set door_open = true
```

- RHS: `true`, `false`, or another bool variable. **No** inline logical
  expressions. **No** compound assignments (`+=` etc.) — meaningless for bool.

## What to cover

- `feature/` — set to literal, set from another bool, several sets in
  sequence, `?` reflects the change.
- `errors/` — RHS isn't bool (e.g. `set door_open = 1`), RHS uses
  `and`/`or`/`not`, RHS uses `+=`/`-=`/`*=`/`/=`, RHS references undeclared
  variable.
- `edge-cases/` — set inside an indented block, set inside a section.

## Reference

`set-integer-variables-compatibility-tests` (done). Compound assignment
ops do NOT apply.
