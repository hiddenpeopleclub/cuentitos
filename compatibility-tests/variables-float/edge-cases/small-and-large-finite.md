# Edge Case: Very Small and Very Large Finite Values

Floats can hold values far smaller and larger than the integer range while
staying finite. These render in plain decimal form (never scientific
notation), with at least one fractional digit. `0.125` (= `1.0 / 8.0`) is an
exactly representable fraction and round-trips precisely.

## Script
```cuentitos
--- variables
float tiny = 0.000001
float huge = 1000000000000000.0
float exact_fraction = 1.0 / 8.0
---

This is the story.
```

## Input
```input
?
s
```

## Result
```result
START
tiny: 0.000001
huge: 1000000000000000.0
exact_fraction: 0.125
This is the story.
END
```
