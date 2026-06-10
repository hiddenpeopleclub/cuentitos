# Require: Two Bool Variables Compared with `=`

The RHS of a bool `req` may reference another bool variable. `req a = b` passes
when both variables hold the same value.

## Script
```cuentitos
--- variables
bool a = true
bool b = true
bool c = false
---

A matches B.
  req a = b
A matches C.
  req a = c
```

## Input
```input
s
```

## Result
```result
START
A matches B.
END
```
