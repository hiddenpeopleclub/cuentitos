# Require Error: Int Literal on the RHS of a String Comparison

A string variable can only be compared against string values. There is no
implicit int-to-string coercion, so comparing it to an integer literal is a
parse-time type error, mirroring the bool rule (see
`variables-bool/errors/compare-bool-with-int-literal.md`).

## Script
```cuentitos
--- variables
string name = "Aria"
---

Line.
  req name = 1
```

## Input
```input
s
```

## Result
```result
require-cross-type-int-literal-rhs.cuentitos:6: ERROR: Type mismatch: cannot compare string 'name' with int '1'.
```
