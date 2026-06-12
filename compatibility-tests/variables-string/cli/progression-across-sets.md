# Query Progression Across Multiple Sets

Multiple `?` queries at different points show the progression of a string
variable's value as successive `set` statements execute.

## Script
```cuentitos
--- variables
string name = "Aria"
---
set name = "Brenn"
First
set name = "Cass"
Second
set name = "Dane"
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
name: "Brenn"
Second
name: "Cass"
Third
name: "Dane"
END
```
