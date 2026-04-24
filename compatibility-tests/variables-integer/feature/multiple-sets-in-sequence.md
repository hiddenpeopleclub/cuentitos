# Multiple Sets in Sequence

Multiple `set` statements in sequence each update the variable. The final
value reflects the last assignment.

## Script
```cuentitos
--- variables
int score = 0
---
set score = 5
set score = 9
set score += 1
Hello
```

## Input
```input
n
n
n
n
?
s
```

## Result
```result
START
Hello
score: 10
END
```
