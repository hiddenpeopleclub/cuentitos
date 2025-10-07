# Jump to END from Root Level

This test verifies that -> END jumps directly to the END block from root level.

## Script
```cuentitos
# Section A
Text in A
-> END

# Section B
Text in B
```

## Input
```input
s
```

## Result
```result
START
-> Section A
Text in A
END
```
