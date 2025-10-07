# Skip in Nested Calls

This test verifies that skip stops at each section boundary in nested calls.

## Script
```cuentitos
# Section A
Line 1 in A
<-> Section B
Back in A

# Section B
Line 1 in B
<-> Section C
Back in B

# Section C
Line 1 in C
Line 2 in C
```

## Input
```input
n,n,n,s,s
```

## Result
```result
Cannot skip - reached the end of the script.
START
-> Section A
Line 1 in A
-> Section B
Line 1 in B
-> Section C
Line 1 in C
Line 2 in C
Back in B
Back in A
-> Section B
Line 1 in B
-> Section C
Line 1 in C
Line 2 in C
Back in B
-> Section C
Line 1 in C
Line 2 in C
END
```
