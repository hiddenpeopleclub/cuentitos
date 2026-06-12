# Set Error: Undeclared Variable on the RHS

A string `set` whose RHS references a variable that was never declared in a
`--- variables` block is a parse-time error: the referenced name cannot
resolve.

## Script
```cuentitos
--- variables
string name = "Aria"
---
set name = ghost
Hello
```

## Input
```input
s
```

## Result
```result
set-undeclared-rhs.cuentitos:4: ERROR: Undefined variable: 'ghost'.
```
