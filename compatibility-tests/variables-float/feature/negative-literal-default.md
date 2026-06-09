# Negative Literal Default

Float defaults may use negative values via unary minus, both on literals and
on parenthesized subexpressions.

## Script
```cuentitos
--- variables
float a = -5.0
float b = -10.5 + 3.0
float c = -(2.5 + 3.0)
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
a: -5.0
b: -7.5
c: -5.5
This is the story.
END
```
