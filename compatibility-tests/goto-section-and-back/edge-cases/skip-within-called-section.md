# Skip Within Called Section

This test verifies that skip (s) only skips to the end of the current called section, not the entire script.

## Script
```cuentitos
# Section A
Line 1 in A
<-> Section B
Back in A

# Section B
Line 1 in B
Line 2 in B
Line 3 in B
```

## Input
```input
n,n,s
```

## Result
```result
START
-> Section A
Line 1 in A
-> Section B
Line 1 in B
Line 2 in B
Line 3 in B
Back in A
-> Section B
Line 1 in B
Line 2 in B
Line 3 in B
END
```
