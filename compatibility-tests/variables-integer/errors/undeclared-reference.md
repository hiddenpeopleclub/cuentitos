# Error: Reference to Undeclared Variable

A default expression cannot reference a variable that is never declared.

## Script
```cuentitos
--- variables
int a = unknown
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
