# Default Using Arithmetic Over Earlier Variables

A float default expression may use `+`, `-`, `*`, `/` and parentheses over
earlier variables and float literals. Division uses standard IEEE behaviour
and does **not** truncate, so `7.0 / 2.0` is `3.5`.

## Script
```cuentitos
--- variables
float a = 5.0
float b = a + 5.0
float c = (a + b) * 2.0
float d = b - a
float e = b / a
float f = 7.0 / 2.0
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
a: 5.0
b: 10.0
c: 30.0
d: 5.0
e: 2.0
f: 3.5
This is the story.
END
```
