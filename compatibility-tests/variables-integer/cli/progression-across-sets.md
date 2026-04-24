# Query Progression Across Multiple Sets

Multiple `?` queries at different points show the progression of a variable's
value as successive `set` statements execute.

## Script
```cuentitos
--- variables
int score = 0
---
set score = 5
First
set score += 10
Second
set score *= 2
Third
```

## Input
```input
n
n
?
n
n
?
n
n
?
s
```

## Result
```result
START
First
score: 5
Second
score: 15
Third
score: 30
END
```
