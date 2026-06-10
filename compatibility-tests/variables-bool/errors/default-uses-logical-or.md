# Error: Logical Operator `or` In A Bool Default

Logical operators (`and`, `or`, `not`) are not allowed in bool variable
defaults. Boolean expressions belong in `req`.

## Script
```cuentitos
--- variables
bool b = true or false
---

This is the story.
```

## Input
```input
s
```

## Result
```result
default-uses-logical-or.cuentitos:2: ERROR: Logical operators (and/or/not) are not allowed in variable defaults; use 'req' for boolean expressions.
```
