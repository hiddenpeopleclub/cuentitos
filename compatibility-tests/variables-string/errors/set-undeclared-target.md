# Set Error: Undeclared Target

A `set` whose target string variable was never declared in a
`--- variables` block is a parse-time error, mirroring the integer rule
(see `variables-integer/errors/set-undeclared-variable.md`).

## Script
```cuentitos
--- variables
string known = "Aria"
---
set unknown = "Brenn"
Hello
```

## Input
```input
s
```

## Result
```result
set-undeclared-target.cuentitos:4: ERROR: Undefined variable: 'unknown'.
```
