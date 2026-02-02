# Error: Require Invalid Syntax

Missing parts in a require statement is a parse-time error.

## Script
```cuentitos
--- variables
int score = 1
---
require score >
Hello
```

## Input
```input
s
```

## Result
```result
Error: Invalid require syntax at line 4
```
