# Query After Set

The `?` CLI command reflects the updated value of a bool variable after a
`set`.

## Script
```cuentitos
--- variables
bool flag = true
---
set flag = false
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
flag: false
END
```
