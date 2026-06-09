# Error: Reserved Keyword `or` Cannot Be Used As A Bool Variable Name

`or` is a reserved logical operator keyword. Declaring a bool variable with
that name is a parse-time error.

## Script
```cuentitos
--- variables
bool or = true
---

This is the story.
```

## Input
```input
s
```

## Result
```result
reserved-keyword-or.cuentitos:2: ERROR: Reserved keyword 'or' cannot be used as a variable name.
```
