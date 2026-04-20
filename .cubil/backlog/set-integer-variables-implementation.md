---
created: 2026-04-20
---

# Set Integer Variables — Implementation

# Set Integer Variables — Implementation

Implement the `set` statement so the compat tests written in "Set Integer
Variables — Compatibility Tests" pass. Part 2 of 2.

## Feature summary

```cuentitos
--- variables
int health = 10
int score
---

# start
  set health = 5
  set health += 1
  set score = health * 2 - 3
```

- Plain assignment: `set <var> = <expr>`
- Compound assignment: `set <var> += <expr>`, `-=`, `*=`, `/=`
- `<expr>` supports:
  - integer literals (including negative)
  - variable references
  - binary operators `+`, `-`, `*`, `/`
  - standard precedence and parentheses
- `set` is a statement block on its own line, following normal indentation
  rules. Valid anywhere a regular block is valid.

## Semantics

- Integer division `/` **truncates** toward zero (e.g. `7 / 2 == 3`).
- Division by zero is a **runtime error**.
- Overflow (e.g. `i64::MAX + 1`) is a **runtime error**.
- After a `set`, the new value is visible to subsequent blocks and to `?`.

## Errors

- Parse-time: reference to undeclared variable on either side of `set`;
  malformed expression.
- Runtime: division by zero; overflow.

## Acceptance

- All compat tests from the paired "Compatibility Tests" task pass.
- Rust unit tests cover the expression parser and evaluator.

## Dependencies

Requires "Set Integer Variables — Compatibility Tests" and "Definition of
Integer Variables — Implementation". Unblocks "Require Integer Variables —
Implementation".
