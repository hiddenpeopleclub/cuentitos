# Error: Negative Literal One Below i64::MIN in a Default

`i64::MIN` (`-9223372036854775808`) is the smallest valid integer literal
in a default. One below it (`-9223372036854775809`) is a parse-time error.
The diagnostic carries the signed literal text so the author can see
exactly which literal exceeds the range — parallel to the matching `set`
and `req` literal-overflow diagnostics.

## Script
```cuentitos
--- variables
int x = -9223372036854775809
---

This is the story.
```

## Input
```input
s
```

## Result
```result
default-i64-min-literal-below.cuentitos:2: ERROR: Integer overflow in default expression for 'x': literal '-9223372036854775809' exceeds the integer range.
```
