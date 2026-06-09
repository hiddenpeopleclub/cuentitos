# Error: Logical Operator `not` In A Bool Default

Logical operators (`and`, `or`, `not`) are not allowed in bool variable
defaults. Boolean expressions belong in `req`.

## Script
```cuentitos
--- variables
bool b = not true
---

This is the story.
```

## Input
```input
s
```

## Result
```result
default-uses-logical-not.cuentitos:2: ERROR: Logical operators (and/or/not) are not allowed in variable defaults; use 'req' for boolean expressions.
```
