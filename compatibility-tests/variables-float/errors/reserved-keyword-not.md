# Error: Reserved Keyword `not` Cannot Be Used as a Variable Name

`not` is a reserved logical operator keyword. Declaring a float variable with
that name in the `--- variables ---` block is a parse-time error.

## Script
```cuentitos
--- variables
float not = 1.5
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
