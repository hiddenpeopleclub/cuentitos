# Error: `freq` With a Non-Numeric Delta

The delta argument of a `freq` line must be a numeric literal. `abc` is not
a number, so this is a parse-time error: the literal's shape is known from
the script text alone.

## Script
```cuentitos
--- variables
enum mood = happy, sad
---
set mood = happy
Something happens.
  (50%) Branch A.
  (50%) Branch B.
    freq mood happy abc
```

## Input
```input
s
```

## Result
```result
freq-non-numeric-delta.cuentitos:8: ERROR: Invalid 'freq' delta: 'abc' is not a number.
```
