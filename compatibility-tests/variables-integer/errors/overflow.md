# Error: Integer Overflow in a Default (Constant-Folded)

Because defaults are evaluated at parse time, integer overflow must be a parse-time error.

## Script
```cuentitos
--- variables
int a = 9223372036854775807 + 1
---

This is the story.
```

## Input
```input
s
```

## Result
```result
overflow.cuentitos:2: ERROR: Integer overflow in default expression for 'a'.
```
