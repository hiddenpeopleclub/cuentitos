# Require: Two Bool Variables Compared with `!=`

`req a != b` passes when the two bool variables hold different values and
fails when they are equal.

## Script
```cuentitos
--- variables
bool a = true
bool b = true
bool c = false
---

A differs from C.
  req a != c
A differs from B.
  req a != b
```

## Input
```input
s
```

## Result
```result
START
A differs from C.
END
```
