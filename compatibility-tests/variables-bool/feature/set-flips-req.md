# Require: `set` Earlier in the Script Flips a Later Bool `req`

A `set` executed before a `req` changes whether that `req` passes. The bool
comparison is evaluated using the variable's current runtime value.

## Script
```cuentitos
--- variables
bool flag = false
---

Before set, flag is false.
  req flag = false
Before set, flag is true.
  req flag = true
set flag = true
After set, flag is false.
  req flag = false
After set, flag is true.
  req flag = true
```

## Input
```input
s
```

## Result
```result
START
Before set, flag is false.
After set, flag is true.
END
```
