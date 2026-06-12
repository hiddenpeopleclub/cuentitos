# Require: `set` Earlier in the Script Flips a Later `req` (Float)

A `set` executed before a float `req` changes whether that `req` passes. The
comparison is evaluated using the variable's current runtime value.

## Script
```cuentitos
--- variables
float flag = 0.0
---

Before set, flag is zero.
  req flag = 0.0
Before set, flag is one.
  req flag = 1.0
set flag = 1.0
After set, flag is zero.
  req flag = 0.0
After set, flag is one.
  req flag = 1.0
```

## Input
```input
s
```

## Result
```result
START
Before set, flag is zero.
After set, flag is one.
END
```
