# Float Variable Declaration Without Default

A float variable declared without a default value should initialize to `0.0`.
Float values always render with at least one fractional digit, so the zero
default prints as `0.0` (distinct from the integer `0`).

## Script
```cuentitos
--- variables
float a_float
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
a_float: 0.0
This is the story.
END
```
