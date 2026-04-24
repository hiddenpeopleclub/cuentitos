# Require: `set` Earlier in the Script Flips a Later `req`

A `set` executed before a `req` changes whether that `req` passes. The
comparison is evaluated using the variable's current runtime value.

## Script
```cuentitos
--- variables
int flag = 0
---

Before set, flag is zero.
  req flag = 0
Before set, flag is one.
  req flag = 1
set flag = 1
After set, flag is zero.
  req flag = 0
After set, flag is one.
  req flag = 1
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
