# Set: Negative Literal on the RHS

A `set` may assign a negative float literal to a variable. The variable's
runtime value is updated to the signed value and reflected by `?`.

## Script
```cuentitos
--- variables
float x = 0.0
---

set x = -7.5
After.
```

## Input
```input
n
?
s
```

## Result
```result
START
After.
x: -7.5
END
```
