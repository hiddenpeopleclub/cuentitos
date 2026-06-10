# Set: Bool Literal `true` Updates the Variable

A `set <var> = true` statement assigns the `true` literal to a declared bool
variable. A subsequent `?` reflects the new value.

## Script
```cuentitos
--- variables
bool door_open = false
---
set door_open = true
Hello
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
Hello
door_open: true
END
```
