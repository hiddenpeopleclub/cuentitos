# Set Error: Malformed RHS Expression

The RHS of a string `set` is a single string literal or a single variable
reference. Trailing tokens after an otherwise-valid literal — here a second
literal with no connector — leave the RHS unparseable, which is a
parse-time error.

## Script
```cuentitos
--- variables
string name = "Aria"
---
set name = "a" "b"
```

## Input
```input
s
```

## Result
```result
set-malformed-expression.cuentitos:4: ERROR: Malformed 'set' statement: '"a" "b"'. ('set' is reserved at the start of a line; indent or rephrase to use it in narrative text.)
```
