# Jump to START from Root Level

This test verifies that -> START jumps back to the START block from root level.

## Script
```cuentitos
# Section A
Text in A
-> START

# Section B
Text in B
```

## Input
```input
n,n,n,n,n,n,q
```

## Result
```result
START
-> Section A
Text in A
START
-> Section A
Text in A
QUIT
```
