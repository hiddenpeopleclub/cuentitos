# Error: Require Unknown Variable

Requiring an undefined variable is a parse-time error.

## Script
```cuentitos
--- variables
int score
---
require lives = 3
Hello
```

## Input
```input
s
```

## Result
```result
Error: Unknown variable name 'lives' at line 4
```
