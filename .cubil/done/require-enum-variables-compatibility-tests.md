---
created: 2026-05-26
---

# Require on enum variables — compatibility tests

Compat tests for `req` against enum variables. TDD.

## Feature summary

Only `=` and `!=`. No ordering on enums.

```cuentitos
--- variables
enum mood = happy, sad, angry
---
set mood = happy
You smile.
  req mood = happy
You don't grieve.
  req mood != sad
```

- Each side: a value from the variable's enum, or another enum variable of
  the SAME enum.
- Combines with `and`/`or`/`not`.
- Reading an unset enum inside a `req` is a RUNTIME error.

## What to cover

- `feature/` — `=`/`!=` against value names, `=`/`!=` between two same-enum
  variables, combination with `and`/`or`/`not`, multiple sibling `req`s,
  req after a set.
- `errors/` — value not in the variable's enum (parse-time), `<`/`>` on enum
  (parse-time), type mismatch across enums (parse-time), reading an unset
  enum in a req (runtime), undeclared variable.
- `edge-cases/` — req on the same enum where two enums in the script share a
  value name (the name is correctly scoped per-variable).

## Reference

`require-integer-variables-compatibility-tests` (done).
