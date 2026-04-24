# Error: Malformed Default Expression

A default expression with a dangling operator should fail to parse.

## Script
```cuentitos
--- variables
int a = 5 +
---

This is the story.
```

## Input
```input
s
```

## Result
```result
malformed-expression.cuentitos:2: ERROR: Malformed default expression: '5 +'.
```
