# Error: Malformed Default Expression

A float default expression with a dangling operator should fail to parse.

## Script
```cuentitos
--- variables
float a = 5.0 +
---

This is the story.
```

## Input
```input
s
```

## Result
```result
malformed-expression.cuentitos:2: ERROR: Malformed default expression: '5.0 +'.
```
