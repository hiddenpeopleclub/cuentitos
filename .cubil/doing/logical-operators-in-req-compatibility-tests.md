---
created: 2026-05-04
---

# Logical Operators in req — Compatibility Tests

# Logical Operators in `req` — Compatibility Tests

Author the compatibility tests for `AND`, `OR`, `NOT` in `req` conditions
**before** the implementation lands. The tests define the spec; the
implementation task makes them pass.

## Context

Follow-up to "Require Integer Variables". Adds `AND`, `OR`, `NOT` (all caps)
to `req` conditions so authors can combine comparisons inline instead of
stacking sibling `req` blocks.

## Syntax to cover

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
- Logical operators combine **comparisons**, not bare integer expressions.
- Implicit AND across sibling `req`s still works and composes with inline
  logical operators.

## Test buckets

Place tests under `compatibility-tests/variables-integer/<bucket>/` using
the existing numbering convention. Match the format of existing tests in
those folders.

### `feature/`
- `AND` of two comparisons (true and false outcomes).
- `OR` of two comparisons (true and false outcomes).
- `NOT` of a comparison (true and false outcomes).
- `NOT` combined with `AND` and `OR`.
- Parenthesized grouping that **changes outcome** vs. default precedence —
  e.g. `(a OR b) AND c` vs. `a OR b AND c` on the same inputs.
- Combination with sibling `req`s — implicit AND still ANDs with an inline
  `OR` block.

### `errors/`
- Lowercase `and`/`or`/`not` used where an operator is expected (parser
  treats them as identifiers and surfaces a clear "expected operator" or
  "undefined variable" error).
- Logical operator applied to a bare integer expression
  (e.g. `req health AND shield > 0`).
- Unbalanced parentheses — opening without closing, closing without
  opening.
- Missing operand on either side of `AND`/`OR`; missing operand after
  `NOT`.

### `edge-cases/`
- Variable literally named `and` / `or` / `not` (lowercase) — still usable
  as an identifier in an arithmetic context.
- Precedence without parens: `a OR b AND c` must behave as
  `a OR (b AND c)`.
- Deeply nested parentheses (3+ levels).

## Format

Each test is a markdown file with this structure:
````markdown
# Test Name

Short description.

## Script
```cuentitos
// script
```

## Input
```input
n,n,s
```

## Result
```result
// expected output
```
````

Match the wording of existing error-message tests exactly — the runtime
prefixes runtime errors and the parser prefixes parse errors in specific
formats already established in `compatibility-tests/variables-integer/`.

## Acceptance

- All new tests fail under `./bin/run-compat` (because the feature isn't
  implemented yet) — that's correct and expected. The implementation task
  will make them pass.
- Tests follow the existing numbering and folder layout.
- Push to a branch and open a PR for review **before** implementation
  starts.

## Dependencies

None — `req` integer support has already landed (PR #73 merged).
