# Set Error: Undeclared Variable in RHS Expression

Referencing a variable that was not declared in the `--- variables ---`
block inside a `set` expression is a parse-time error.

## Script
```cuentitos
--- variables
int score
---
set score = health + 1
Hello
```

## Input
```input
s
```

## Result
```result
expression-undeclared-variable.cuentitos:4: ERROR: Undefined variable: 'health'.
```
