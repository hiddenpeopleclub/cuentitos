# Require Error: Undeclared Variable on the LHS of a String Comparison

A `req` whose LHS references a variable that was never declared is a
parse-time error, even when the RHS is a valid string literal.

## Script
```cuentitos
--- variables
string name = "Aria"
---

Line.
  req missing = "x"
```

## Input
```input
s
```

## Result
```result
require-undeclared-variable-lhs.cuentitos:6: ERROR: Undefined variable: 'missing'.
```
