# Require Error: Undeclared Variable on the RHS of a String Comparison

A string `req` whose RHS is a bare identifier referencing a variable that was
never declared is a parse-time error.

## Script
```cuentitos
--- variables
string name = "Aria"
---

Line.
  req name = other
```

## Input
```input
s
```

## Result
```result
require-undeclared-variable-rhs.cuentitos:6: ERROR: Undefined variable: 'other'.
```
