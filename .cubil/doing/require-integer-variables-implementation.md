---
created: 2026-04-20
---

# Require Integer Variables — Implementation

# Require Integer Variables — Implementation

Implement the `req` statement so the compat tests written in "Require Integer
Variables — Compatibility Tests" pass. Part 2 of 2.

## Feature summary

`req` is a **child block** of the block it gates. If the condition is false
at runtime, the **parent block and all its descendants are silently skipped**.

```cuentitos
--- variables
int health = 10
---

# start
  You are alive.
    req health > 0
  You're hurt but moving.
    req health > 0
    req health < 5
```

Multiple `req` siblings act as implicit AND.

## Condition grammar

```
req <var> <op> <expr>
```

- `<op>`: `>`, `<`, `>=`, `<=`, `=`, `!=`
- `<expr>`: any integer expression (same grammar as RHS of `set`) — literals,
  variable references, `+ - * /`, parentheses.
- No logical operators (`and`/`or`/`not`) in this iteration.

## Semantics

- When a `req` fails: parent block is skipped, along with all of its children
  (including any other `req` siblings; short-circuit is fine but not
  required).
- `?` and CLI navigation commands behave as if the skipped blocks were not
  present in the output.
- Arithmetic inside `<expr>` follows the same runtime rules as `set`
  (truncating division, runtime error on divide-by-zero and overflow).

## Errors (parse-time)

- Referencing a variable (LHS, RHS, or inside the expression) that was never
  declared in `--- variables`.
- Unknown comparison operator.
- Malformed expression.
- `req` at the top level (must have a parent block to gate).

## Acceptance

- All compat tests from the paired "Compatibility Tests" task pass.
- Rust unit tests cover the condition parser and evaluator.

## Dependencies

Requires "Require Integer Variables — Compatibility Tests", "Definition of
Integer Variables — Implementation", and "Set Integer Variables —
Implementation".
