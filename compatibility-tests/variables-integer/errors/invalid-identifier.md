# Error: Invalid Variable Identifier

A variable name that starts with a digit should be rejected.

## Script
```cuentitos
--- variables
int 2foo = 1
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
