# Error: Self-Reference in a Default

A float default expression cannot reference the variable it is declaring. The
name is not yet in scope on its own declaration line, so this is reported as a
forward reference.

## Script
```cuentitos
--- variables
float a = a
---

This is the story.
```

## Input
```input
s
```

## Result
```result
self-reference-in-default.cuentitos:2: ERROR: Forward reference: variable 'a' referenced before declaration.
```
