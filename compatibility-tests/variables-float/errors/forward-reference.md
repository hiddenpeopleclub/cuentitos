# Error: Forward Reference in a Default

A float default expression cannot reference a variable declared later in the
same block.

## Script
```cuentitos
--- variables
float a = b
float b = 5.0
---

This is the story.
```

## Input
```input
s
```

## Result
```result
forward-reference.cuentitos:2: ERROR: Forward reference: variable 'b' referenced before declaration.
```
