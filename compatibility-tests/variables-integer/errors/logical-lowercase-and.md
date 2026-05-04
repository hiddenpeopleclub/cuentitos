# Logical Operator Error: Lowercase `and` Is Not an Operator

Logical operators are uppercase only. A lowercase `and` is parsed as an
identifier. Because no variable named `and` is declared, the parser reports
an undefined-variable error.

## Script
```cuentitos
--- variables
int x = 5
int y = 3
---

Line.
  req x > 0 and y > 0
```

## Input
```input
s
```

## Result
```result
logical-lowercase-and.cuentitos:7: ERROR: Undefined variable: 'and'.
```
