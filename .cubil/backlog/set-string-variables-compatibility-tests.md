---
created: 2026-05-26
---

# Set on string variables — compatibility tests

Compat tests for `set` on string variables. TDD.

## Feature summary

```cuentitos
--- variables
string name = "Aria"
---
set name = "Brenn"
set name = other_string_var
```

- RHS: a string literal or a reference to another string variable. **No**
  concatenation (separate task). **No** compound assignment.

## What to cover

- `feature/` — set to literal, set from another string, several sets in
  sequence, `?` reflects the change.
- `errors/` — RHS isn't a string, RHS uses `+`, RHS uses compound assignment,
  RHS references undeclared variable.
- `edge-cases/` — set to empty string, set to a string containing escapes,
  set inside section / indented block.

## Reference

`set-integer-variables-compatibility-tests` (done).
