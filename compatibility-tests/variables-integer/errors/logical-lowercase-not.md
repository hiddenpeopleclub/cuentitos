# Logical Operator Error: Lowercase `not` Is Not an Operator

Logical operators are uppercase only. A lowercase `not` placed where the
prefix operator `NOT` would belong is parsed as an identifier. Because no
variable named `not` is declared, the parser reports an undefined-variable
error rather than a generic malformed-expression error.

## Script
```cuentitos
--- variables
int x = 5
int y = 3
---

Line.
  req x > 0 AND not y > 0
```

## Input
```input
s
```

## Result
```result
logical-lowercase-not.cuentitos:7: ERROR: Undefined variable: 'not'.
```
