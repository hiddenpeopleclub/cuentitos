# Error: Reserved Keyword `or` Cannot Be Used as a Variable Name

`or` is a reserved logical operator keyword. Declaring a variable with
that name in the `--- variables ---` block is a parse-time error.

## Script
```cuentitos
--- variables
int or = 5
---

Hello.
```

## Input
```input
s
```

## Result
```result
reserved-keyword-or.cuentitos:2: ERROR: Reserved keyword 'or' cannot be used as a variable name.
```
