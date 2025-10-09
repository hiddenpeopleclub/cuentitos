# CLI Error: Section Not Found

This test verifies that CLI shows error for nonexistent section and continues.

## Script
```cuentitos
# Section A
Text in A

# Section B
Text in B
```

## Input
```input
n
n
-> Fake Section
n
s
```

## Result
```result
ERROR: Section not found: Fake Section
START
-> Section A
Text in A
-> Section B
Text in B
END
```
