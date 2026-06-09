# Error: Duplicate Variable Name

Declaring the same variable name twice in a `--- variables` block should
fail, regardless of type.

## Script
```cuentitos
--- variables
string a
string b = "bee"
string a = "again"
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
