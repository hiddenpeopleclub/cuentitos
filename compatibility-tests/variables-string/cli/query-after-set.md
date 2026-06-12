# Query After Set

The `?` CLI command reflects the updated value of a string variable after a
`set`, re-rendered double-quoted.

## Script
```cuentitos
--- variables
string name = "Aria"
---
set name = "Brenn"
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
name: "Brenn"
END
```
