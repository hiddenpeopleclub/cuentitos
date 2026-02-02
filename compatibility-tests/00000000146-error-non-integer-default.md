# Error: Non-Integer Default Value

Assigning a non-integer value to an int variable is a parse-time error.

## Script
```cuentitos
--- variables
int score = hello
---
Hello
```

## Input
```input
s
```

## Result
```result
Error: Invalid integer value 'hello' for variable 'score' at line 2
```
