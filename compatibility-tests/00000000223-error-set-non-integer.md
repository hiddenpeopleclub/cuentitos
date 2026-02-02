# Error: Set Non-Integer Value

Assigning a non-integer value in a set statement is a parse-time error.

## Script
```cuentitos
--- variables
int score
---
set score = hello
Hello
```

## Input
```input
s
```

## Result
```result
Error: Invalid integer value 'hello' for variable 'score' at line 4
```
