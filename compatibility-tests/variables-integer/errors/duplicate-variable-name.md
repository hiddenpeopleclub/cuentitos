# Error: Duplicate Variable Name

Declaring the same variable name twice in a `--- variables` block should fail.

## Script
```cuentitos
--- variables
int a
int b = 1
int a = 2
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
