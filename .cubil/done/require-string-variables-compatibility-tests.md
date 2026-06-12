---
created: 2026-05-26
---

# Require on string variables — compatibility tests

Compat tests for `req` against string variables. TDD.

## Feature summary

Only `=` and `!=` on strings. Ordering and substring matching are NOT
supported in v1 (substring is its own follow-up task).

```cuentitos
--- variables
string name = "Aria"
---
You are Aria.
  req name = "Aria"
You are not Brenn.
  req name != "Brenn"
```

- Each side: string literal or string variable reference.
- Combines with `and`/`or`/`not`.

## What to cover

- `feature/` — `=`/`!=` against literal, `=`/`!=` between two string
  variables, combination with `and`/`or`/`not`, multiple sibling `req`s,
  req after a set.
- `errors/` — `<`/`>`/`<=`/`>=` on strings (parse-time error), type
  mismatch (string vs int/bool/float), undeclared variable, substring keyword
  if attempted (point to the follow-up task in the error message? — decide).
- `edge-cases/` — empty-string equality, escapes equal-compared, comparison
  inside nested req branch.

## Reference

`require-integer-variables-compatibility-tests` (done). Substring matching
lives in `substring-matching-in-req-for-strings` (future).
