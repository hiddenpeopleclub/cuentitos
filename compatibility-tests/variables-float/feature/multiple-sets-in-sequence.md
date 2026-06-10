# Multiple Sets in Sequence

Multiple `set` statements in sequence each update the variable. The final
value reflects the last assignment, mixing plain and compound forms.

## Script
```cuentitos
--- variables
float score = 0.0
---
set score = 5.0
set score = 9.0
set score += 1.5
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
score: 10.5
END
```
