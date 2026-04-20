---
created: 2026-04-20
---

# Require Integer Variables — Compatibility Tests

Write the compatibility tests that define how the `req` statement should
behave. Part 1 of 2 for "Require Integer Variables" (the implementation lives
in a sibling task).

## Feature summary (for test authoring)

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

Multiple `req` siblings act as implicit AND — all must pass for the parent to
be shown.

### Condition grammar

```
req <var> <op> <expr>
```

- `<op>`: `>`, `<`, `>=`, `<=`, `=`, `!=`
- `<expr>`: integer literals, variable references, `+ - * /`, parentheses
  (same grammar as RHS of `set`).
- No logical operators (`and`/`or`/`not`) in this iteration — use multiple
  `req` children for AND.

### Semantics

- When a `req` fails: parent block is skipped, along with all of its children
  (including any other `req` siblings).
- `?` and CLI navigation behave as if the skipped blocks were not present in
  the output.
- Arithmetic in `<expr>` follows the same runtime rules as `set` (truncating
  division, runtime error on divide-by-zero and overflow).

## Tests to write

Organized under `compatibility-tests/variables-integer/<bucket>/`:

- `feature/`:
  - one test per comparison operator (`>`, `<`, `>=`, `<=`, `=`, `!=`)
  - `req` gating a single-line block
  - `req` gating a block with children (children also skipped)
  - multiple `req` siblings as implicit AND (all pass → shown; one fails →
    parent skipped)
  - `req` with a variable on the RHS
  - `req` with arithmetic on the RHS
  - interaction with `set` (a `set` earlier in the script flips whether a
    later `req` passes)
- `cli/`:
  - `?` output after a block is skipped by a failing `req` (state still
    reflects actual variable values, not the skipped block)
- `errors/`:
  - parse-time error: `req` referencing an undeclared variable (LHS, RHS, or
    inside the expression)
  - parse-time error: `req` at the top level (no parent block)
  - parse-time error: unknown comparison operator
  - parse-time error: malformed expression
  - runtime error: division by zero inside a `req` expression
  - runtime error: overflow inside a `req` expression
- `edge-cases/`:
  - `req` as the only child of a block
  - `req` alongside non-`req` siblings (non-`req` children still skipped when
    a `req` fails)
  - nested gated blocks (child block has its own `req` under a parent that
    also has a `req`)

## Acceptance

- All tests above exist under `compatibility-tests/variables-integer/` following the
  repo convention.
- `./bin/run-compat` runs; the new tests **fail** (expected until the
  implementation task is done).

## Dependencies

Requires "Definition of Integer Variables — Compatibility Tests" and "Set
Integer Variables — Compatibility Tests" for shared conventions. Paired with
"Require Integer Variables — Implementation".
