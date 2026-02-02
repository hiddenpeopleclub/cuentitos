# Require Integer Variable Fail

A failing requirement stops execution and reports an error.

## Script
```cuentitos
--- variables
int lives = 2
---
require lives >= 3
Hello
```

## Input
```input
n
n
```

## Result
```result
START
ERROR: Requirement failed: lives >= 3 (actual: 2) at line 4
END
```
