# Error: Require Invalid Operator

Using an invalid operator in a require statement is a parse-time error.

## Script
```cuentitos
--- variables
int score = 5
---
require score >< 3
Hello
```

## Input
```input
s
```

## Result
```result
Error: Invalid require operator '><' at line 4
```
