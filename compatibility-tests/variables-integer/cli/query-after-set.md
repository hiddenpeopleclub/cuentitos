# Query After Set

The `?` CLI command reflects the updated value of a variable after a `set`.

## Script
```cuentitos
--- variables
int health = 10
---
set health = 3
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
health: 3
END
```
