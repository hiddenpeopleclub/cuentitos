# Require: Arithmetic Expression on the RHS

The right-hand side of a `req` comparison may be an arithmetic expression —
using literals, variables, `+ - * /`, and parentheses — evaluated at runtime
before the comparison.

## Script
```cuentitos
--- variables
int x = 10
int y = 3
---

x is greater than y plus five.
  req x > y + 5
x is greater than y times ten.
  req x > y * 10
x equals parenthesized sum.
  req x = (y + 7)
x equals integer division result.
  req x = 21 / 2
```

## Input
```input
s
```

## Result
```result
START
x is greater than y plus five.
x equals parenthesized sum.
x equals integer division result.
END
```
