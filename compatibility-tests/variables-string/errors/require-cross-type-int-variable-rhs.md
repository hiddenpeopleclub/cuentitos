# Require Error: Int Variable on the RHS of a String Comparison

A string variable can only be compared against string values. Referencing an
int variable on the RHS of a string `req` is a parse-time type error — there
is no implicit coercion between the two types.

## Script
```cuentitos
--- variables
int count = 3
string name = "Aria"
---

Line.
  req name = count
```

## Input
```input
s
```

## Result
```result
require-cross-type-int-variable-rhs.cuentitos:7: ERROR: Type mismatch: cannot compare string 'name' with int 'count'.
```
