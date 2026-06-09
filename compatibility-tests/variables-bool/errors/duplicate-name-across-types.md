# Error: Duplicate Variable Name Across Types

Declaring `bool x` after `int x` (or vice versa) in the same `--- variables`
block is a duplicate-name error; types do not create separate namespaces.

## Script
```cuentitos
--- variables
int x
bool x = true
---

This is the story.
```

## Input
```input
s
```

## Result
```result
duplicate-name-across-types.cuentitos:3: ERROR: Duplicate variable name: 'x' already declared. Previously declared at line 2.
```
