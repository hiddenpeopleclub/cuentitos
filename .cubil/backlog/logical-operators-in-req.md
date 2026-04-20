---
created: 2026-04-20
---

# Logical Operators in req

# Logical Operators in `req`

Follow-up to "Require Integer Variables". Adds `AND`, `OR`, `NOT` (all caps)
to `req` conditions so authors can combine comparisons inline instead of
stacking sibling `req` blocks.

## Motivation

The first iteration of `req` only supports a single comparison per block;
multiple sibling `req`s act as implicit AND. That covers the common case but
forces awkward block structure for OR and for any NOT. All-caps keywords
mirror the visual weight of `req` itself and stay unambiguous next to
identifiers.

## Syntax

```cuentitos
--- variables
int health = 10
int shield = 0
---

# start
  You're defended.
    req health > 0 AND shield > 0
  You're exposed but alive.
    req health > 0 AND NOT shield > 0
  You can act.
    req health > 0 OR shield > 0
```

- Keywords: `AND`, `OR`, `NOT` — uppercase only. Lowercase `and`/`or`/`not`
  must remain valid as identifiers / regular words.
- Precedence (tightest first): `NOT` → `AND` → `OR`.
- Parentheses group sub-expressions:
  `req (health > 0 OR shield > 0) AND health < 100`.
- A `req` condition is still a boolean expression composed of comparisons
  (`>`, `<`, `>=`, `<=`, `=`, `!=`); logical operators combine comparisons,
  they do not apply to bare integer expressions.
- Implicit AND across sibling `req`s still works — the two styles compose.

## Semantics

- Short-circuit evaluation left-to-right is fine but not required for
  correctness. (If a runtime error — e.g. divide-by-zero — sits inside a
  branch the runtime would not have needed to evaluate, short-circuiting
  avoids the error; document whichever behavior is implemented.)
- All runtime arithmetic rules from `set`/`req` still apply to comparison
  operands.

## Errors (parse-time)

- Lowercase `and`/`or`/`not` used where a logical operator is expected
  (parser should treat them as identifiers and surface a clearer "expected
  operator" error).
- Mixing a bare integer expression with a logical operator
  (e.g. `req health AND shield > 0`) — logical operators require boolean
  operands.
- Unbalanced parentheses.
- Missing operand on either side of `AND`/`OR`; missing operand after `NOT`.

## Acceptance

- Compat tests under `compatibility-tests/variables-integer/<bucket>/`:
  - `feature/`: `AND` of two comparisons, `OR` of two comparisons, `NOT` of
    a comparison, `NOT` combined with `AND`/`OR`, parenthesized grouping
    that changes outcome vs. default precedence, combination with sibling
    `req`s (implicit AND still ANDs with an inline `OR`).
  - `errors/`: lowercase `and`/`or`/`not` where an operator is expected,
    logical operator applied to bare integer expression, unbalanced
    parentheses, missing operand around `AND`/`OR`/`NOT`.
  - `edge-cases/`: variable literally named `and`/`or`/`not` (lowercase) —
    still usable as an identifier; precedence without parens
    (`a OR b AND c` must behave as `a OR (b AND c)`); deeply nested
    parentheses.
- Rust unit tests for the updated condition parser and evaluator, including
  precedence and short-circuit behavior.

## Dependencies

Requires "Require Integer Variables — Implementation".
