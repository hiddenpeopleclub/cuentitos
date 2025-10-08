# Jump to RESTART from Root Level

This test verifies that -> RESTART clears state and jumps to START from root level.

## Script
```cuentitos
# Section A
Text in A
-> RESTART

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
