# Error: Reserved Keyword `and` Cannot Be Used as a Variable Name

`and` is a reserved logical operator keyword. Declaring a variable with
that name in the `--- variables ---` block is a parse-time error.

## Script
```cuentitos
--- variables
int and = 5
---

Hello.
```

## Input
```input
s
```

## Result
```result
reserved-keyword-and.cuentitos:2: ERROR: Reserved keyword 'and' cannot be used as a variable name.
```
