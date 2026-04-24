# Set Error: Malformed RHS Expression

A `set` whose RHS is a syntactically incomplete expression — here, a
trailing `+` with no right operand — is a parse-time error.

## Script
```cuentitos
--- variables
int x = 0
---

set x = 5 +
```

## Input
```input
s
```

## Result
```result
set-malformed-expression.cuentitos:5: ERROR: Malformed expression in 'set': '5 +'.
```
