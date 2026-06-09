# Error: Cross-Type Reference in a Float Default

A float default expression may reference only float literals and earlier float
variables. There is no implicit int-to-float coercion, so referencing an int
variable in a float default is a parse-time type-mismatch error.

## Script
```cuentitos
--- variables
int count = 3
float ratio = count * 2.0
---

This is the story.
```

## Input
```input
s
```

## Result
```result
cross-type-default.cuentitos:3: ERROR: Type mismatch: default for float ratio must be a float expression, but count is int.
```
