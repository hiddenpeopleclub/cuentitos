# Set: Negative Literal on the RHS

A `set` may assign a negative integer literal to a variable. The variable's
runtime value is updated to the signed value and reflected by `?`.

## Script
```cuentitos
--- variables
int x = 0
---

set x = -7
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
x: -7
END
```
