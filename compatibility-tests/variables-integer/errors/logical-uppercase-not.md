# Logical Operator Error: Uppercase `NOT` Is Not an Operator

Logical operators are lowercase only. Uppercase `NOT` placed where the
prefix operator `not` would belong is parsed as an identifier. Because no
variable named `NOT` is declared, the parser reports an undefined-variable
error.

## Script
```cuentitos
--- variables
int x = 5
int y = 3
---

Line.
  req x > 0 and NOT y > 0
```

## Input
```input
s
```

## Result
```result
logical-uppercase-not.cuentitos:7: ERROR: Undefined variable: 'NOT'.
```
