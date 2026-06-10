# Query After Set

The `?` CLI command reflects the updated value of a float variable after a
`set`.

## Script
```cuentitos
--- variables
float health = 10.0
---
set health = 3.5
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
health: 3.5
END
```
