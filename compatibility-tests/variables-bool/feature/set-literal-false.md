# Set: Bool Literal `false` Updates the Variable

A `set <var> = false` statement assigns the `false` literal to a declared bool
variable, overwriting a `true` default. A subsequent `?` reflects the new
value.

## Script
```cuentitos
--- variables
bool door_open = true
---
set door_open = false
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
door_open: false
END
```
