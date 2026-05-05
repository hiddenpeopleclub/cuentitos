---
created: 2026-05-04
---

# Logical Operators in `req` — Compatibility Tests

Author the compatibility tests for `and`, `or`, `not` in `req` conditions
**before** the implementation lands. The tests define the spec; the
implementation task makes them pass.

> **Status:** PR #74 open with 25 tests on `cuentitos-logical-ops-compat`.
> Spec was revised mid-flight from uppercase `AND`/`OR`/`NOT` to
> lowercase `and`/`or`/`not` (see commit `d25d275`). This document
> reflects the final spec.

## Context

Follow-up to "Require Integer Variables". Adds `and`, `or`, `not`
(lowercase, matching the rest of the keyword family `set`/`req`/`int`) to
`req` conditions so authors can combine comparisons inline instead of
stacking sibling `req` blocks.

## Syntax to cover

```cuentitos
--- variables
int health = 10
int shield = 0
---

# start
  You're defended.
    req health > 0 and shield > 0
  You're exposed but alive.
    req health > 0 and not shield > 0
  You can act.
    req health > 0 or shield > 0
```

- Keywords: `and`, `or`, `not` — lowercase only. Uppercase `AND`/`OR`/`NOT`
  are not keywords and parse as ordinary (undefined) identifiers.
- `and`/`or`/`not` are reserved — they cannot be used as variable names.
- Precedence (tightest first): `not` → `and` → `or`.
- Parentheses group sub-expressions:
  `req (health > 0 or shield > 0) and health < 100`.
- Logical operators combine **comparisons**, not bare integer expressions.
- Implicit AND across sibling `req`s still works and composes with inline
  logical operators.

## Test buckets

Place tests under `compatibility-tests/variables-integer/<bucket>/` using
descriptive kebab-case filenames (no numeric prefix). Match the format of
existing tests in those folders.

### `feature/`
- `and` of two comparisons (true and false outcomes).
- `or` of two comparisons (true and false outcomes).
- `not` of a comparison (true and false outcomes).
- `not` combined with `and` and `or`.
- Parenthesized grouping that **changes outcome** vs. default precedence —
  e.g. `(a or b) and c` vs. `a or b and c` on the same inputs.
- Combination with sibling `req`s — implicit AND still ANDs with an inline
  `or` block.

### `errors/`
- Uppercase `AND`/`OR`/`NOT` used where an operator is expected (parser
  treats them as identifiers and surfaces `Undefined variable: 'AND'.`).
- Logical operator applied to a bare integer expression
  (e.g. `req health and shield > 0`).
- Unbalanced parentheses — opening without closing, closing without
  opening.
- Missing operand on either side of `and`/`or`; missing operand after
  `not`.
- `and`/`or`/`not` declared as a variable name (e.g. `int and = 5`)
  rejected as a reserved-keyword error.

### `edge-cases/`
- Precedence without parens: `a or b and c` must behave as
  `a or (b and c)`.
- Deeply nested parentheses (3+ levels).

## Format

Conventions are defined in the `compatibility-tests` skill at
`.claude/skills/compatibility-tests/SKILL.md` — file format, input
commands, output rules, error format, one-outcome-per-test.

## Acceptance

- All new tests fail under `./bin/run-compat` (because the feature isn't
  implemented yet) — that's correct and expected.
- 228 existing tests continue to pass — no regressions.
- Push to `cuentitos-logical-ops-compat` and open a PR for review
  **before** implementation starts.

## Dependencies

None — `req` integer support has already landed (PR #73 merged).
