# Mutual Recursion

This test verifies that mutual recursion works with the call stack (A calls B, B calls A).

## Script
```cuentitos
# Section A
In A
<-> Section B
Back in A final

# Section B
In B
```

## Input
```input
s
```

## Result
```result
START
-> Section A
In A
-> Section B
In B
Back in A final
-> Section B
In B
END
```
