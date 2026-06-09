# Error: Reserved Keyword `not` Cannot Be Used As A Bool Variable Name

`not` is a reserved logical operator keyword. Declaring a bool variable with
that name is a parse-time error.

## Script
```cuentitos
--- variables
bool not = true
---

This is the story.
```

## Input
```input
s
```

## Result
```result
reserved-keyword-not.cuentitos:2: ERROR: Reserved keyword 'not' cannot be used as a variable name.
```
