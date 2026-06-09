# Error: Duplicate Variable Name

Declaring the same variable name twice in a `--- variables` block should fail,
reporting the line of the earlier declaration.

## Script
```cuentitos
--- variables
float a
float b = 1.5
float a = 2.5
---

This is the story.
```

## Input
```input
s
```

## Result
```result
duplicate-variable-name.cuentitos:4: ERROR: Duplicate variable name: 'a' already declared. Previously declared at line 2.
```
