# Multiple Sets in Sequence

Multiple `set` statements in sequence each update the variable. The final
value reflects the last assignment.

## Script
```cuentitos
--- variables
string name = "Aria"
---
set name = "Brenn"
set name = "Cass"
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
name: "Cass"
END
```
