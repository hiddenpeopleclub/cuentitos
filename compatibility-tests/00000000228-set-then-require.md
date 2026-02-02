# Set Then Require

A require statement uses the updated value from a previous set.

## Script
```cuentitos
--- variables
int score = 1
---
set score = 5
require score >= 5
Hello
```

## Input
```input
s
```

## Result
```result
START
Hello
END
```
