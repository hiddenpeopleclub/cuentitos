# Self Reference

A `set` expression can reference the variable being assigned.

## Script
```cuentitos
--- variables
int counter = 5
---
set counter = counter + 1
Hello
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
Hello
counter: 6
END
```
