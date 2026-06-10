# Require: Bool Compared to a Literal with `!=`

A `req` may use `!=` to compare a bool variable against `true` or `false`.
The comparison passes when the variable is not equal to the literal.

## Script
```cuentitos
--- variables
bool flag = true
---

Flag is not false.
  req flag != false
Flag is not true.
  req flag != true
```

## Input
```input
s
```

## Result
```result
START
Flag is not false.
END
```
