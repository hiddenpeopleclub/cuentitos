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
set-malformed-expression.cuentitos:5: ERROR: Malformed 'set' statement: '5 +'. ('set' is reserved at the start of a line; indent or rephrase to use it in narrative text.)
```
