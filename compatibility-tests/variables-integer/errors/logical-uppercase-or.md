# Logical Operator Error: Uppercase `OR` Is Not an Operator

Logical operators are lowercase only. Uppercase `OR` is parsed as an
identifier. Because no variable named `OR` is declared, the parser
reports an undefined-variable error.

## Script
```cuentitos
--- variables
int x = 5
int y = 3
---

Line.
  req x > 0 OR y > 0
```

## Input
```input
s
```

## Result
```result
logical-uppercase-or.cuentitos:7: ERROR: Undefined variable: 'OR'.
```
