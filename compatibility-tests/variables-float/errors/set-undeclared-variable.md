# Set Error: Undeclared Target

A `set` whose target variable was never declared in a `--- variables`
block is a parse-time error.

## Script
```cuentitos
--- variables
float known = 0.0
---

set unknown = 1.0
```

## Input
```input
s
```

## Result
```result
set-undeclared-variable.cuentitos:5: ERROR: Undefined variable: 'unknown'.
```
