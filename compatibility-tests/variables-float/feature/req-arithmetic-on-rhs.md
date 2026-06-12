# Require: Arithmetic Expression on the RHS (Float)

The right-hand side of a float `req` comparison may be an arithmetic
expression — using float literals, variables, `+ - * /`, and parentheses —
evaluated at runtime before the comparison. Float division does **not**
truncate, so `21.0 / 2.0` is `10.5`, not `10`.

## Script
```cuentitos
--- variables
float x = 10.5
float y = 3.0
---

x is greater than y plus five.
  req x > y + 5.0
x is greater than y times ten.
  req x > y * 10.0
x equals parenthesized sum.
  req x = (y + 7.5)
x equals non-truncating division result.
  req x = 21.0 / 2.0
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
x equals non-truncating division result.
END
```
