# Logical Operator Error: Uppercase `AND` Is Not an Operator

Logical operators are lowercase only. Uppercase `AND` is parsed as an
identifier. Because no variable named `AND` is declared, the parser
reports an undefined-variable error.

## Script
```cuentitos
--- variables
int x = 5
int y = 3
---

Line.
  req x > 0 AND y > 0
```

## Input
```input
s
```

## Result
```result
logical-uppercase-and.cuentitos:7: ERROR: Undefined variable: 'AND'.
```
