# Require: `set` Earlier in the Script Flips a Later String `req`

A `set` executed before a `req` changes whether that `req` passes. The
comparison is evaluated using the string variable's current runtime value.

## Script
```cuentitos
--- variables
string flag = "off"
---

Before set, flag is off.
  req flag = "off"
Before set, flag is on.
  req flag = "on"
set flag = "on"
After set, flag is off.
  req flag = "off"
After set, flag is on.
  req flag = "on"
```

## Input
```input
s
```

## Result
```result
START
Before set, flag is off.
After set, flag is on.
END
```
