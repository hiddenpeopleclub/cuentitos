# CLI Error: Navigate Above Root

This test verifies that CLI shows error when trying to navigate above root level.

## Script
```cuentitos
# Section A
Text in A
```

## Input
```input
n
n
-> ..
n
s
```

## Result
```result
START
-> Section A
Text in A
ERROR: Cannot navigate above root level
END
Cannot skip - reached the end of the script.
```
