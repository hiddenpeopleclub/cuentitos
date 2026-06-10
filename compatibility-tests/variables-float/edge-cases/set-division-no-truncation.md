# Edge Case: Set Division Does Not Truncate

Float division in a `set` expression follows IEEE semantics and does **not**
truncate toward zero the way integer division does. `7.0 / 2.0` yields `3.5`,
and `1.0 / 4.0` yields the exactly representable `0.25`. This is the float
counterpart to the integer `edge-cases/division-truncation.md` test.

## Script
```cuentitos
--- variables
float a = 0.0
float b = 0.0
---
set a = 7.0 / 2.0
set b = 1.0 / 4.0
Hello
```

## Input
```input
n
?
s
```

## Result
```result
START
Hello
a: 3.5
b: 0.25
END
```
