# Require Edge Case: Inexact Binary Representation in Comparisons

Most decimal fractions are not exactly representable in IEEE-754 binary, so
`0.1 + 0.2` stores `0.30000000000000004`, which is **not** equal to the
nearest `f64` to `0.3`. A `req` comparison uses the exact stored values, so
`sum = 0.3` fails, `sum != 0.3` passes, and `sum > 0.3` passes — the stored
sum is fractionally larger than `0.3`.

## Script
```cuentitos
--- variables
float sum = 0.1 + 0.2
---

Sum is not exactly three tenths.
  req sum != 0.3
Sum is greater than three tenths.
  req sum > 0.3
Sum is exactly three tenths.
  req sum = 0.3
```

## Input
```input
s
```

## Result
```result
START
Sum is not exactly three tenths.
Sum is greater than three tenths.
END
```
