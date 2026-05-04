# Logical Operator Error: Lowercase `or` Is Not an Operator

Logical operators are uppercase only. A lowercase `or` is parsed as an
identifier. Because no variable named `or` is declared, the parser reports
an undefined-variable error.

## Script
```cuentitos
--- variables
int x = 5
int y = 3
---

Line.
  req x > 0 or y > 0
```

## Input
```input
s
```

## Result
```result
logical-lowercase-or.cuentitos:7: ERROR: Undefined variable: 'or'.
```
