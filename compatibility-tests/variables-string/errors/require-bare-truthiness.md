# Require Error: Bare String Variable Is Not a Truthiness Shortcut

The bare `req <name>` truthiness shortcut is bool-only; strings have no
notion of truthiness. A bare string operand with no comparison operator is a
malformed `req` expression, matching the int behavior (see
`variables-integer` — a bare int operand is likewise rejected).

## Script
```cuentitos
--- variables
string name = "Aria"
---

Line.
  req name
```

## Input
```input
s
```

## Result
```result
require-bare-truthiness.cuentitos:6: ERROR: Malformed expression in 'req': 'name'.
```
