# Nested Calls (Two Levels)

This test verifies that nested calls work like a call stack - A calls B, B calls C, returns to B, returns to A.

## Script
```cuentitos
# Section A
In A
<-> Section B
Back in A

# Section B
In B
<-> Section C
Back in B

# Section C
In C
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
-> Section C
In C
Back in B
Back in A
-> Section B
In B
-> Section C
In C
Back in B
-> Section C
In C
END
```
