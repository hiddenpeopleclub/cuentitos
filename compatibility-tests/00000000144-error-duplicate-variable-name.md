# Error: Duplicate Variable Name

Defining two variables with the same name is a parse-time error.

## Script
```cuentitos
--- variables
int score
int score = 5
---
Hello
```

## Input
```input
s
```

## Result
```result
Error: Duplicate variable name 'score' at line 3
```
