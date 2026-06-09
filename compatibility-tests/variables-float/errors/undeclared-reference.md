# Error: Reference to Undeclared Variable

A float default expression cannot reference a variable that is never declared.
This differs from a forward reference (which names a variable declared later in
the same block): `unknown` is never declared at all.

## Script
```cuentitos
--- variables
float a = unknown
---

This is the story.
```

## Input
```input
s
```

## Result
```result
undeclared-reference.cuentitos:2: ERROR: Undefined variable: 'unknown'.
```
