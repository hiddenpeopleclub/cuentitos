# Query Progression Across Multiple Sets

Multiple `?` queries at different points show the progression of an enum
variable's value as successive `set` statements assign different variants.

## Script
```cuentitos
--- variables
enum mood = happy, sad, angry
---
set mood = happy
First
set mood = sad
Second
set mood = angry
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
mood: happy
Second
mood: sad
Third
mood: angry
END
```
