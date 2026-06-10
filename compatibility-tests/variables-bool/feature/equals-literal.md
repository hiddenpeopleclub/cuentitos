# Require: Bool Compared to a Literal with `=`

A `req` may compare a bool variable against the literals `true` or `false`
using `=`. The comparison passes when the variable equals the literal.

## Script
```cuentitos
--- variables
bool flag = true
---

Flag is true.
  req flag = true
Flag is false.
  req flag = false
```

## Input
```input
s
```

## Result
```result
START
Flag is true.
END
```
