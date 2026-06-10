# Query Progression Across Multiple Sets

Multiple `?` queries at different points show the progression of a bool
variable's value as successive `set` statements flip it.

## Script
```cuentitos
--- variables
bool flag = false
---
set flag = true
First
set flag = false
Second
set flag = true
Third
```

## Input
```input
n
?
n
?
n
?
s
```

## Result
```result
START
First
flag: true
Second
flag: false
Third
flag: true
END
```
