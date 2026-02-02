# Error: Set Invalid Syntax

Missing `=` in a set statement is a parse-time error.

## Script
```cuentitos
--- variables
int score
---
set score 5
Hello
```

## Input
```input
s
```

## Result
```result
Error: Invalid set syntax at line 4
```
