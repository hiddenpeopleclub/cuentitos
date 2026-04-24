---
created: 2026-04-20
---

# Set Integer Variables — Compatibility Tests

Write the compatibility tests that define how the `set` statement should
behave. Part 1 of 2 for "Set Integer Variables" (the implementation lives in a
sibling task).

## Feature summary (for test authoring)

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
- Compound assignment: `+=`, `-=`, `*=`, `/=`
- `<expr>`: integer literals (including negative), variable references,
  binary `+ - * /`, parentheses, standard precedence.
- `set` is a statement block on its own line, following normal indentation
  rules. Valid anywhere a regular block is valid.

### Semantics

- Integer division `/` truncates toward zero (e.g. `7 / 2 == 3`).
- Division by zero is a **runtime error**.
- Overflow is a **runtime error**.
- After a `set`, the new value is visible to subsequent blocks and to `?`.

## Tests to write

Organized under `compatibility-tests/variables-integer/<bucket>/`:

- `feature/`:
  - plain `set`
  - each compound op (`+=`, `-=`, `*=`, `/=`)
  - expression with multiple variables and precedence
  - parentheses
  - negative literals
  - self-reference (`set x = x + 1`)
  - `set` inside nested blocks / sections
  - `set` appearing multiple times in sequence
- `cli/`:
  - `?` reflecting updated values after a `set`
  - `?` at multiple points showing progression across several `set`s
- `errors/`:
  - parse-time error: `set` on an undeclared variable
  - parse-time error: expression references an undeclared variable
  - parse-time error: malformed expression
  - runtime error: division by zero at a non-constant `set`
  - runtime error: overflow at a non-constant `set`
- `edge-cases/`:
  - division truncation toward zero (including with negative operands)

## Acceptance

- All tests above exist under `compatibility-tests/variables-integer/` following the
  repo convention.
- `./bin/run-compat` runs; the new tests **fail** (expected until the
  implementation task is done).

## Dependencies

Requires "Definition of Integer Variables — Compatibility Tests" conventions
to be established (variable declaration syntax). Paired with "Set Integer
Variables — Implementation".
