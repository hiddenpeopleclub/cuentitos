# Chain of Jumps in Called Section

This test verifies that a chain of regular jumps within a called section are all resolved before returning.

## Script
```cuentitos
# Section A
In A
<-> Section B
Back in A

# Section B
In B
-> Section C

# Section C
In C
-> Section D

# Section D
In D
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
Back in A
-> Section B
In B
-> Section C
In C
-> Section D
In D
END
```
