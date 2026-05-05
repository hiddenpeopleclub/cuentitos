---
created: 2026-05-04
---

# Logical Operators in `req` — Implementation

Implement `and`, `or`, `not` in `req` conditions so the compatibility
tests from the **Compatibility Tests** task (PR #74) pass.

## Context

Follow-up to "Require Integer Variables". Adds lowercase logical operators
to `req` conditions so authors can combine comparisons inline instead of
stacking sibling `req` blocks. Casing matches the rest of the keyword
family (`set`, `req`, `int`).

## Syntax

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
  Either side or both, but not too much.
    req (health > 0 or shield > 0) and health < 100
```

## Grammar

- Keywords: `and`, `or`, `not` — lowercase only and reserved (cannot be
  used as variable names). Uppercase `AND`/`OR`/`NOT` are not keywords;
  they parse as ordinary identifiers (and surface as
  `Undefined variable: 'AND'.` if used in a `req` condition).
- Precedence (tightest first): `not` → comparison → `and` → `or`.
- Parentheses group sub-expressions.
- A `req` condition is a boolean expression composed of comparisons
  (`>`, `<`, `>=`, `<=`, `=`, `!=`); logical operators combine boolean
  values (results of comparisons or other logical expressions), not bare
  integer expressions.
- Implicit AND across sibling `req`s still works and composes with inline
  logical operators.

## Semantics

- Short-circuit evaluation left-to-right. If a runtime error
  (e.g. divide-by-zero) sits in a branch the runtime didn't need to
  evaluate, short-circuiting avoids the error.
- All runtime arithmetic rules from `set`/`req` continue to apply to
  comparison operands.

## Suggested data model

The current `RequirementStatement` is `{ left: Expression, operator:
ComparisonOperator, right: Expression }`. Logical operators turn the `req`
condition into a tree. Recommended shape:

```rust
pub enum BooleanExpression {
    Comparison(RequirementStatement),
    And(Box<BooleanExpression>, Box<BooleanExpression>),
    Or(Box<BooleanExpression>, Box<BooleanExpression>),
    Not(Box<BooleanExpression>),
}
```

The `Block::Requirement` variant then carries a `BooleanExpression`
instead of a single `RequirementStatement`. Migrate existing single-
comparison parsing to wrap the result in `BooleanExpression::Comparison`.

This keeps the symmetry that already exists between `BinaryOperator::apply`
on `Value`s and `ComparisonOperator::apply` returning `Result<bool,
EvaluationError>`. `BooleanExpression::evaluate` returns `Result<bool,
EvaluationError>`.

You may pick a different shape if you have a strong reason — but document
the choice.

## Parser changes

- Tokenize lowercase `and`/`or`/`not` as logical-operator tokens (in `req`
  condition context). Uppercase variants stay as identifier tokens.
- Add `and`/`or`/`not` to the reserved-keyword check used by the variable
  declaration parser.
- Recursive descent for the boolean expression:
  `parse_or → parse_and → parse_not → parse_primary`
  where `parse_primary` is either a parenthesized boolean expression or a
  comparison (single `req` statement).
- Parse-time validation:
  - Each leaf must be a comparison; reject a bare integer expression where
    a boolean is expected.
  - Each side of `and`/`or` must be present; `not` must have an operand.
  - Parentheses must balance.
- Surface clear errors — exact wording is pinned by the compat tests on
  PR #74.

## Runtime changes

- `evaluate_requirement_gating` already iterates sibling `req` blocks with
  short-circuit AND. Add a `BooleanExpression::evaluate` that handles the
  three combinators with short-circuit semantics.
- `RuntimeError` may need a new "expected boolean got integer" variant if
  any error path can reach the runtime — but parse-time validation should
  rule that out, so this is likely an `unreachable!()` with a meaningful
  message.

## Acceptance

- All 25 compatibility tests from PR #74 pass under `./bin/run-compat`.
- New Rust unit tests for the boolean-expression parser and evaluator,
  including precedence and short-circuit behavior.
- `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`,
  `cargo test`, `./bin/run-compat` all clean.
- No regressions — the existing 228 compat tests continue to pass.

## Dependencies

Requires PR #74 (`cuentitos-logical-ops-compat`) to have landed (or at
least be visible on a branch you can rebase against).
