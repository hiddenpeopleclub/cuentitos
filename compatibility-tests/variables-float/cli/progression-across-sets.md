# Query Progression Across Multiple Sets

Multiple `?` queries at different points show the progression of a float
variable's value as successive `set` statements execute.

## Script
```cuentitos
--- variables
float score = 0.0
---
set score = 5.0
First
set score += 10.0
Second
set score *= 2.0
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
score: 5.0
Second
score: 15.0
Third
score: 30.0
END
```
