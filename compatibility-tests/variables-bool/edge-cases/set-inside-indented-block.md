# Set Inside an Indented Block

A `set` statement is valid anywhere a regular block is valid, including nested
under an indented parent block. The `set` executes silently as the runtime
advances through its parent's children, and a later `?` reflects the change.

## Script
```cuentitos
--- variables
bool flag = false
---
Outer.
  set flag = true
  Inner.
```

## Input
```input
n
n
?
s
```

## Result
```result
START
Outer.
Inner.
flag: true
END
```
