# Edge Case: Inexact Binary Representation

Most decimal fractions are not exactly representable in IEEE-754 binary, so a
sum like `0.1 + 0.2` is not exactly `0.3`. The engine renders the true stored
value using its shortest decimal form that round-trips back to the same `f64`
(never scientific notation), which for `0.1 + 0.2` is `0.30000000000000004`.
A plain `0.3` literal stores the nearest `f64` to `0.3`, whose shortest
round-tripping form is `0.3`, so the two are visibly distinct.

## Script
```cuentitos
--- variables
float sum = 0.1 + 0.2
float third = 1.0 / 3.0
float plain = 0.3
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
sum: 0.30000000000000004
third: 0.3333333333333333
plain: 0.3
This is the story.
END
```
