# Require Edge Case: Division in a Condition Does Not Truncate

Float division inside a `req` expression follows IEEE semantics and does
**not** truncate toward zero the way integer division does. `7.0 / 2.0`
evaluates to `3.5`, so a `req` comparing against `3.5` passes while one
comparing against the truncated `3.0` fails. This is the `req` counterpart to
`edge-cases/set-division-no-truncation.md`.

## Script
```cuentitos
--- variables
float result = 7.0 / 2.0
---

Result equals the exact half.
  req result = 3.5
Result is strictly between three and four.
  req result > 3.0 and result < 4.0
Result equals the truncated whole number.
  req result = 3.0
```

## Input
```input
s
```

## Result
```result
START
Result equals the exact half.
Result is strictly between three and four.
END
```
