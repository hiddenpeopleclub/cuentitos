---
created: 2026-04-20
---

# Definition of Integer Variables — Implementation

Implement integer variable **declarations** so the compat tests under
`compatibility-tests/variables-integer/` (merged in PR #68) pass. Part 2 of 2.

## Feature summary

Support a `--- variables` block at the top of a script that declares integer
variables with optional default values.

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
  Expressions use the same grammar as the RHS of `set`.
- Defaults are evaluated **once**, in declaration order, at start of script.
  Because every default can only reference literals and earlier defaults, the
  entire value is **constant-foldable at parse time**: the parser must
  evaluate eagerly and surface arithmetic problems (division by zero,
  overflow) as **parse-time errors**.
- Integer division `/` truncates toward zero.
- Variables declared here are **global to the script**.

## `?` debug input

`?` prints every declared variable and its current value, one per line, in
declaration order. Must work at any point during execution and reflect
current values.

If `?` is used when no variables are declared (no `--- variables` block, or
an empty one), the runtime emits a warning
`<filename>.cuentitos:0: WARNING: No variables declared.` (line `0` is the
sentinel for CLI-triggered warnings, matching the existing warning format).

## Errors (parse-time)

- Duplicate variable name
- Invalid identifier
- Malformed default expression
- Default expression references a variable not yet declared (forward
  reference) or that does not exist
- Division by zero in a default expression (constant-folded)
- Overflow in a default expression (constant-folded, including when the
  overflow occurs through a reference to a previously-declared variable —
  see `errors/overflow-through-variable.md`)
- Malformed `--- variables` / `---` delimiters

## Acceptance

- All compat tests from the paired "Compatibility Tests" task pass.
- Rust unit tests for the parser cover the same error cases.

## Dependencies

Compat tests shipped in PR #68. Unblocks "Set Integer Variables —
Implementation" and "Require Integer Variables — Implementation".
