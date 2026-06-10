# Multiple Bool Sets in Sequence

Multiple `set` statements in sequence each update the bool variable. The final
value reflects the last assignment.

## Script
```cuentitos
--- variables
bool flag = false
---
set flag = true
set flag = false
set flag = true
Done.
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
Done.
flag: true
END
```
