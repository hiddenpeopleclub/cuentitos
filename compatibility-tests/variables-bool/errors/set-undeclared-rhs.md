# Set Error: Undeclared Variable on the RHS

A bool `set` whose RHS references a variable that was never declared in a
`--- variables` block is a parse-time error: the referenced name cannot
resolve.

## Script
```cuentitos
--- variables
bool door_open = false
---
set door_open = ghost
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
