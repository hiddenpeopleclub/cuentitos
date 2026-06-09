# Error: Reserved Keyword `or` Cannot Be Used as a Variable Name

`or` is a reserved logical operator keyword. Declaring a float variable with
that name in the `--- variables ---` block is a parse-time error.

## Script
```cuentitos
--- variables
float or = 1.5
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
