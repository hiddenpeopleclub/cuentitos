# Set Error: Comparison Operator on the RHS

The RHS of a `set` is an arithmetic expression (`+ - * /`, parentheses,
variables, literals). Comparison operators such as `>` belong to `req`
conditions, not to `set` assignments, so a comparison on the RHS of a `set`
is a parse-time error.

## Script
```cuentitos
--- variables
float health = 10.0
---
set health = health > 5.0
Hello
```

## Input
```input
s
```

## Result
```result
set-comparison-operator-rhs.cuentitos:4: ERROR: Malformed 'set' statement: 'health > 5.0'. ('set' is reserved at the start of a line; indent or rephrase to use it in narrative text.)
```
