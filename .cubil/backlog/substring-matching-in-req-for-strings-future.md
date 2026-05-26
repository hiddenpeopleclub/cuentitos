---
created: 2026-05-26
---

# Substring matching in req for strings (future)

Future work: add a substring-matching operator for strings inside `req`.

The initial string req tests / impl explicitly exclude this (only `=` and
`!=`). This task is the placeholder for the follow-up.

## Sketch

Likely shape, to be confirmed when picked up:

```cuentitos
--- variables
string message = "the door is locked"
---
Locked.
  req message contains "locked"
```

- Operator name: probably `contains` (lowercase, like `and`/`or`/`not`).
- Operand types: both sides must be strings.
- Composes with `and`/`or`/`not` like the other comparison operators.
- Decide: case-sensitive only? `not message contains "x"` shape? Empty
  substring semantics?

## When to start

- After the string compat tests + impl tasks have landed and the rest of
  the v1 type set is comfortable.
- Open a design discussion (or a short ADR) before writing compat tests,
  since the operator name and semantics are user-visible.

## References

- `require-string-variables-compatibility-tests` (the v1 req-on-string
  scope that explicitly excludes substring matching).
- `logical-operators-in-req-compatibility-tests` (done) for the pattern of
  introducing a new req keyword.
