# Set Error: Compound Assignment on a String

Compound assignment operators (`+=`, `-=`, `*=`, `/=`) are arithmetic
shortcuts that have no meaning for a string variable — and `+=` would imply
concatenation, which is not supported in v1. Using one on a string `set` is
a parse-time error, mirroring the bool rule (see
`variables-bool/errors/set-compound-assignment.md`).

## Script
```cuentitos
--- variables
string name = "Aria"
---
set name += "Brenn"
Hello
```

## Input
```input
s
```

## Result
```result
set-compound-assignment.cuentitos:4: ERROR: Compound assignment ('+=') is not supported for string variables.
```
