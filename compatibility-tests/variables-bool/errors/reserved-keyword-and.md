# Error: Reserved Keyword `and` Cannot Be Used As A Bool Variable Name

`and` is a reserved logical operator keyword. Declaring a bool variable with
that name is a parse-time error.

## Script
```cuentitos
--- variables
bool and = true
---

This is the story.
```

## Input
```input
s
```

## Result
```result
reserved-keyword-and.cuentitos:2: ERROR: Reserved keyword 'and' cannot be used as a variable name.
```
