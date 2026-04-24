---
created: 2026-04-20
---

# Definition of Integer Variables — Compatibility Tests

Write the compatibility tests that define how integer variable **declarations**
should behave. Part 1 of 2 for "Definition of Integer Variables" (the
implementation lives in a sibling task).

Follow TDD: these tests should be written first and expected to fail until the
implementation task lands.

## Feature summary (for test authoring)

A `--- variables` block at the top of a script declares integer variables with
optional default values.

```cuentitos
--- variables
int an_integer
int an_integer_that_starts_with_one = 1
int five = 5
int another_integer = five + 5
---
```

- Declaration form: `int <name>` (defaults to `0`)
- Declaration with default: `int <name> = <expr>`
- `<name>`: letters, digits, underscores; must not start with a digit.
- `<expr>`: integer literals (including negative), references to variables
  declared **earlier in the same block**, binary `+ - * /`, parentheses.
- Defaults are evaluated once, in declaration order, at start of script.
  Because every default can only reference literals and earlier defaults, the
  whole value is **constant-foldable at parse time**: the parser evaluates
  eagerly; division by zero and overflow in a default are **parse-time
  errors**.
- Integer division `/` truncates toward zero.
- Variables declared here are global to the script.

## `?` debug input

`?` prints every declared variable and its current value, one per line, in
declaration order:

```
an_integer: 0
an_integer_that_starts_with_one: 1
```

`?` must work at any point during execution.

If `?` is used when no variables are declared (either no `--- variables`
block exists, or the block is empty), the runtime emits a warning
(`WARNING: No variables declared` or similar, following the existing warning
format convention in `compatibility-tests/*/edge-cases/*-warning.md`).

## Tests to write

Organized under `compatibility-tests/variables-integer/<bucket>/`:

- `feature/`:
  - no-default declaration
  - declaration with literal default
  - default referencing an earlier variable
  - default using arithmetic over earlier variables
  - multiple variables in one block
  - negative-literal default
- `cli/`:
  - `?` immediately after load
  - `?` after navigating through some blocks (values still reflect initial
    defaults since no `set` exists yet)
- `errors/`:
  - duplicate variable name
  - invalid identifier
  - forward reference in a default
  - reference to an undeclared variable in a default
  - malformed default expression
  - division by zero in a default (constant-folded)
  - overflow in a default (constant-folded)
  - malformed `--- variables` / `---` delimiters
- `edge-cases/`:
  - empty `--- variables` block (`?` emits "no variables declared" warning)
  - nested parentheses in a default
  - `?` before any `---` block exists in the script (emits "no variables
    declared" warning)

## Acceptance

- All tests above exist under `compatibility-tests/variables-integer/` following the
  repo convention.
- `./bin/run-compat` runs; the new tests **fail** (expected until the
  implementation task is done).

## Dependencies

None. Paired with "Definition of Integer Variables — Implementation".
