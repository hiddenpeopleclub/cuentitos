---
created: 2026-05-26
---

# Definition of string variables — compatibility tests

Compat tests for declaring string variables. TDD.

## Feature summary

```cuentitos
--- variables
string name
string hero_name = "Aria"
string echo = hero_name
---
```

- `string <name>` — defaults to `""` (empty string).
- `string <name> = "<literal>"`.
- Literal syntax: double-quoted. Supported escapes: `\"`, `\n`, `\\`. No
  multi-line literals. No interpolation.
- Default RHS allowed: a string literal or a reference to a previously-
  declared string variable. **No** concatenation in v1 (separate task).

## What to cover

- `feature/` — declaration with/without default, ref to earlier string,
  `?` output format (decide and document — e.g. `name: "Aria"`), each
  supported escape.
- `errors/` — unterminated literal, invalid escape (`\q`), multi-line literal
  (literal `\n` in the source mid-line — should error), forward reference,
  duplicate name, reserved keyword, default uses `+` (no concat in v1).
- `edge-cases/` — empty string default, escape-only string (`"\n"`), mixing
  with int/bool/float in the same block.

## Reference

Mirror `compatibility-tests/variables-integer/`.
