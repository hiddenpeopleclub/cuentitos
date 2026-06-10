---
created: 2026-05-26
---

# Require on bool variables — compatibility tests

Compat tests for `req` against bool variables. TDD.

## Feature summary

```cuentitos
--- variables
bool door_open = false
---
The door is open.
  req door_open
The door is closed.
  req not door_open
```

- `req <bool_var>` — truthiness shortcut (passes iff value is `true`).
- `req <bool_var> = true|false`, `req <bool_var> != <other_bool>`.
- Combines with existing `and`/`or`/`not` already implemented for `req`.
- Ordering operators (`< > <= >=`) on bools are a parse-time error.

## What to cover

- `feature/` — truthiness shortcut, `=`/`!=` against literals, `=`/`!=`
  between two bools, combinations with `and`/`or`/`not`, multiple sibling
  `req`s, req after a set.
- `errors/` — bool vs non-bool comparison, `<`/`>`/`<=`/`>=` on bools,
  req against undeclared bool.
- `edge-cases/` — req on bool nested under another req-gated block.

## Reference

`require-integer-variables-compatibility-tests` (done) and
`logical-operators-in-req-compatibility-tests` (done) for the and/or/not interplay.
