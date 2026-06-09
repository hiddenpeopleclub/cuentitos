# Error: Invalid Variable Identifier

A float variable name that starts with a digit should be rejected.

## Script
```cuentitos
--- variables
float 2foo = 1.0
---

This is the story.
```

## Input
```input
s
```

## Result
```result
invalid-identifier.cuentitos:2: ERROR: Invalid variable name: '2foo'. Variable names must start with a letter or underscore.
```
